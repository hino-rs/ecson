//! TLS設定の構築ヘルパーモジュール。
//!
//! 開発用の自己署名証明書の生成と、
//! PEMファイルからの本番用TLS設定の構築を提供します。

use rustls::{
    ServerConfig,
    pki_types::{CertificateDer, PrivateKeyDer},
};
use std::{path::Path, sync::Arc};
use tokio_rustls::TlsAcceptor;

/// PEMファイルから本番用の TlsAcceptor を構築します。
///
/// # 引数
/// * `cert_path` - 証明書ファイルのパス（例: "/etc/letsencrypt/live/example.com/fullchain.pem"）
/// * `key_path`  - 秘密鍵ファイルのパス（例: "/etc/letsencrypt/live/example.com/privkey.pem"）
pub fn build_tls_acceptor(
    cert_path: impl AsRef<Path>,
    key_path: impl AsRef<Path>,
) -> Result<TlsAcceptor, Box<dyn std::error::Error + Send + Sync>> {
    // 証明書を読み込む
    let cert_file = std::fs::read(cert_path)?;
    let certs: Vec<CertificateDer<'static>> =
        rustls_pemfile::certs(&mut cert_file.as_slice()).collect::<Result<_, _>>()?;

    // 秘密鍵を読み込む
    let key_file = std::fs::read(key_path)?;
    let key =
        rustls_pemfile::private_key(&mut key_file.as_slice())?.ok_or("秘密鍵が見つかりません")?;

    make_acceptor(certs, key)
}

/// rcgen で自己署名証明書を生成し、TlsAcceptor を返します（開発用）。
///
/// 生成した証明書はメモリ上にのみ存在し、ディスクには書き込みません。
pub fn build_self_signed_acceptor(
    subject_alt_names: impl Into<Vec<String>>,
) -> Result<TlsAcceptor, Box<dyn std::error::Error + Send + Sync>> {
    let san = subject_alt_names.into();

    // rcgen で証明書と秘密鍵を生成
    let cert = rcgen::generate_simple_self_signed(san)?;

    // DER形式に変換して rustls の型に渡す
    let cert_der = CertificateDer::from(cert.cert.der().to_vec());
    let key_der = PrivateKeyDer::try_from(cert.signing_key.serialize_der())
        .map_err(|e| format!("秘密鍵変換エラー: {e}"))?;

    make_acceptor(vec![cert_der], key_der)
}

/// 証明書と秘密鍵から TlsAcceptor を組み立てる共通ロジック
fn make_acceptor(
    certs: Vec<CertificateDer<'static>>,
    key: PrivateKeyDer<'static>,
) -> Result<TlsAcceptor, Box<dyn std::error::Error + Send + Sync>> {
    let config = ServerConfig::builder()
        .with_no_client_auth() // クライアント証明書は要求しない（一般的な用途）
        .with_single_cert(certs, key)?;

    Ok(TlsAcceptor::from(Arc::new(config)))
}
