//! Helper module for building TLS configurations.
//!
//! Provides self-signed certificate generation for development use
//! and production TLS setup from PEM files.

use rustls::{
    ServerConfig,
    pki_types::{CertificateDer, PrivateKeyDer},
};
use std::{path::Path, sync::Arc};
use tokio_rustls::TlsAcceptor;

/// Builds a production-ready `TlsAcceptor` from PEM files.
///
/// # Arguments
/// * `cert_path` - Path to the certificate file (e.g. "/etc/letsencrypt/live/example.com/fullchain.pem")
/// * `key_path`  - Path to the private key file (e.g. "/etc/letsencrypt/live/example.com/privkey.pem")
pub fn build_tls_acceptor(
    cert_path: impl AsRef<Path>,
    key_path: impl AsRef<Path>,
) -> Result<TlsAcceptor, Box<dyn std::error::Error + Send + Sync>> {
    let cert_file = std::fs::read(cert_path)?;
    let certs: Vec<CertificateDer<'static>> =
        rustls_pemfile::certs(&mut cert_file.as_slice()).collect::<Result<_, _>>()?;

    let key_file = std::fs::read(key_path)?;
    let key =
        rustls_pemfile::private_key(&mut key_file.as_slice())?.ok_or("private key not found")?;

    make_acceptor(certs, key)
}

/// Generates a self-signed certificate with rcgen and returns a `TlsAcceptor` (for development use).
///
/// The generated certificate exists only in memory and is never written to disk.
pub fn build_self_signed_acceptor(
    subject_alt_names: impl Into<Vec<String>>,
) -> Result<TlsAcceptor, Box<dyn std::error::Error + Send + Sync>> {
    let san = subject_alt_names.into();

    let cert = rcgen::generate_simple_self_signed(san)?;

    let cert_der = CertificateDer::from(cert.cert.der().to_vec());
    let key_der = PrivateKeyDer::try_from(cert.signing_key.serialize_der())
        .map_err(|e| format!("private key conversion error: {e}"))?;

    make_acceptor(vec![cert_der], key_der)
}

/// Shared logic to assemble a `TlsAcceptor` from a certificate and private key.
fn make_acceptor(
    certs: Vec<CertificateDer<'static>>,
    key: PrivateKeyDer<'static>,
) -> Result<TlsAcceptor, Box<dyn std::error::Error + Send + Sync>> {
    let config = ServerConfig::builder()
        .with_no_client_auth() // no client certificate required (standard usage)
        .with_single_cert(certs, key)?;

    Ok(TlsAcceptor::from(Arc::new(config)))
}
