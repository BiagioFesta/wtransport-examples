use anyhow::Result;
use base64::engine::general_purpose::STANDARD as Base64Engine;
use base64::Engine;
use rcgen::CertificateParams;
use rcgen::DistinguishedName;
use rcgen::DnType;
use rcgen::KeyPair;
use rcgen::PKCS_ECDSA_P256_SHA256;
use ring::digest::digest;
use ring::digest::SHA256;
use time::Duration;
use time::OffsetDateTime;

#[derive(Clone)]
pub struct SelfCertificate {
    /// DER certificate.
    pub certificate: Vec<u8>,

    /// DER private key.
    pub key: Vec<u8>,

    /// Base64 SHA256 public key.
    pub fingerprint: String,
}

/// Generates a self-signed certificate for WebTransport connections.
pub fn generate_certificate<S: AsRef<str>>(common_name: S) -> Result<SelfCertificate> {
    let keypair = KeyPair::generate(&PKCS_ECDSA_P256_SHA256)?;
    let digest = digest(&SHA256, &keypair.public_key_der());
    let fingerprint = Base64Engine.encode(digest);

    let mut dname = DistinguishedName::new();
    dname.push(DnType::CommonName, common_name.as_ref());

    let mut cert_params = CertificateParams::new(vec![common_name.as_ref().to_string()]);
    cert_params.distinguished_name = dname;
    cert_params.alg = &PKCS_ECDSA_P256_SHA256;
    cert_params.key_pair = Some(keypair);
    cert_params.not_before = OffsetDateTime::now_utc()
        .checked_sub(Duration::days(5))
        .unwrap();
    cert_params.not_after = OffsetDateTime::now_utc()
        .checked_add(Duration::days(5))
        .unwrap();

    let certificate = rcgen::Certificate::from_params(cert_params)?;

    Ok(SelfCertificate {
        certificate: certificate.serialize_der()?,
        key: certificate.serialize_private_key_der(),
        fingerprint,
    })
}
