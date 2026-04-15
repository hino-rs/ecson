//! A module that handles individual TCP client connections and mediates message exchange with ECS.
//!
//! VarInt is a variable-length integer encoding that uses 1 to 5 bytes.
//! If the most significant bit(MSB) of each bute is set to 1, the following byte also contains valid data.
//! For sending payloads to ECS, use `NetworkPayload::Binary(Vec<u8>)` and pass the raw byte sequence for
//! one packet(including the packet ID and data, but not the length field) directly.

use ecson_ecs::channels::{NetworkEvent, NetworkPayload};
use std::fmt::Formatter;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

/// Maximum number of bytes for VarInt
const VARINT_MAX_BYTES: usize = 5;

/// Maximum number of bytes per packet. Protection against receiving abnormally large data.
const MAX_PACKET_SIZE: usize = 2 * 1024 * 1024; // 2MB

// =========================================================
// Entry Point
// =========================================================

pub async fn handle_connection(
    stream: TcpStream,
    conn_id: u64,
    ecs_tx: mpsc::Sender<NetworkEvent>,
    client_buffer: usize,
) {
    info!("TCP connection established for ID {conn_id}");

    let (client_tx, client_rx) = mpsc::channel::<NetworkPayload>(client_buffer);

    // Connect to ECS Notification
    if ecs_tx
        .send(NetworkEvent::Connected {
            id: conn_id,
            sender: client_tx,
        })
        .await
        .is_err()
    {
        warn!("ECS channel closed before Connected could be sent (ID: {conn_id})");
        return;
    }

    // Splits TCP streams into read and write operations
    let (reader, writer) = stream.into_split();
    let cancel = CancellationToken::new();

    // Write task: ECS -> client
    let write_task = tokio::spawn(write_loop(writer, client_rx, cancel.clone(), conn_id));

    // Read task: client ->
    let read_task = tokio::spawn(read_loop(reader, ecs_tx.clone(), cancel, conn_id));

    let _ = tokio::join!(write_task, read_task);
    info!("TCP connection closed for ID {conn_id}");
}

// =========================================================
// Receive Loop
// =========================================================

async fn read_loop(
    mut reader: tokio::net::tcp::OwnedReadHalf,
    ecs_tx: mpsc::Sender<NetworkEvent>,
    cancel: CancellationToken,
    conn_id: u64,
) {
    loop {
        // Read packet length(VarInt)
        let packet_len = match read_varint_async(&mut reader).await {
            Ok(n) => n as usize,
            Err(e) => {
                // EOF or Read Error → As Disconnected
                if !matches!(e, VarIntError::Eof) {
                    warn!("ID {conn_id}: failed to readd packet length: {e:?}");
                }
                break;
            }
        };

        // Size Validation(rejects abnormally large packets)
        if packet_len == 0 || packet_len > MAX_PACKET_SIZE {
            warn!("ID {conn_id}: invalid packet size: {packet_len}, disconnecting");
            break;
        }

        // Read packet body in one go
        let mut buf = vec![0u8; packet_len];
        if let Err(e) = reader.read_exact(&mut buf).await {
            warn!("ID {conn_id}: failed to read packet body: {e}");
            break;
        }

        // Send as binary payload to ECS(interpretation delegated to ECS's System)
        if ecs_tx
            .send(NetworkEvent::Message {
                id: conn_id,
                payload: NetworkPayload::Binary(buf),
            })
            .await
            .is_err()
        {
            break;
        }

        // End receive loop → Cancel send task as well
        cancel.cancel();
        let _ = ecs_tx
            .send(NetworkEvent::Disconnected { id: conn_id })
            .await;
    }
}

// =========================================================
// Send Loop
// =========================================================

async fn write_loop(
    mut writer: tokio::net::tcp::OwnedWriteHalf,
    mut client_rx: mpsc::Receiver<NetworkPayload>,
    cancel: CancellationToken,
    conn_id: u64,
) {
    loop {
        tokio::select! {
            _ = cancel.cancelled() => break,
            payload = client_rx.recv() => {
                let data = match payload {
                    Some(NetworkPayload::Binary(b)) => b,
                    // TCP servers should only use binary data.
                    // If text data is received, it should be sent as a UTF-8 byte sequence.
                    Some(NetworkPayload::Text(t)) => t.into_bytes(),
                    None => break, // Channel closed
                };

                // Here, data = Packet ID + payload (the raw byte sequence assembled by the ECS)
                let len_prefix = encode_varint(data.len() as i32);

                if let Err(e) = writer.write_all(&len_prefix).await {
                    error!("ID {conn_id}: failed to write length prefix: {e}");
                    break;
                }
                if let Err(e) = writer.write_all(&data).await {
                    error!("ID {conn_id}: failed to write packet body: {e}");
                    break;
                }
            }
        }
    }
}

// =========================================================
// VarInt Utilities
// =========================================================

/// VarInt Read Error
#[derive(Debug)]
pub enum VarIntError {
    /// Connection closed normally
    Eof,
    /// I/O Error
    Io(std::io::Error),
    /// Exceeded 5 bytes
    TooLong,
}

impl std::fmt::Display for VarIntError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Eof => write!(f, "EOF"),
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::TooLong => write!(f, "VarInt exceeds 5 bytes"),
        }
    }
}

/// Asynchronously reads a single VarInt from a TCP stream.
///
/// If the most significant bit (bit7) of any byte is set to 1, the following byte also contains part of the value.
pub async fn read_varint_async(
    reader: &mut tokio::net::tcp::OwnedReadHalf,
) -> Result<i32, VarIntError> {
    let mut result = 0i32;
    let mut shift = 0u32;

    for _ in 0..VARINT_MAX_BYTES {
        let mut byte = [0u8; 1];
        match reader.read_exact(&mut byte).await {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                return Err(VarIntError::Eof);
            }
            Err(e) => return Err(VarIntError::Io(e)),
        }

        // Shift the lower 7 bits and incorporate them into the result
        let b = byte[0];
        result |= ((b & 0x7f) as i32) << shift;

        // Terminate if MSB is 0
        if b & 0x80 == 0 {
            return Ok(result);
        }

        shift += 7;
    }

    Err(VarIntError::TooLong)
}

/// Encodes an `i32` value into a VarInt byte sequence.
///
/// The smaller the value, the fewer bytes it occupies (1 to 5 bytes).
pub fn encode_varint(mut value: i32) -> Vec<u8> {
    let mut buf = Vec::with_capacity(5);
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7; // Arithmetic shift with sign preservation
        if value != 0 {
            byte |= 0x80; // Set the MSB to indicate "more data follows"
        }
        buf.push(byte);
        if value == 0 {
            break;
        }
    }
    buf
}

// =========================================================
// Tests
// =========================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_varint_single_byte() {
        // 0〜127は1バイトで表現できる
        assert_eq!(encode_varint(0), vec![0x00]);
        assert_eq!(encode_varint(1), vec![0x01]);
        assert_eq!(encode_varint(127), vec![0x7F]);
    }

    #[test]
    fn test_encode_varint_multi_byte() {
        // 128 = 0x80 → [0x80, 0x01]
        assert_eq!(encode_varint(128), vec![0x80, 0x01]);
        // 255 = 0xFF → [0xFF, 0x01]
        assert_eq!(encode_varint(255), vec![0xFF, 0x01]);
        // 2097151 (最大3バイト) → [0xFF, 0xFF, 0x7F]
        assert_eq!(encode_varint(2097151), vec![0xFF, 0xFF, 0x7F]);
    }
}
