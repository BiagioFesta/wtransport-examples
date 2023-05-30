use crate::certificate::SelfCertificate;
use anyhow::Context;
use anyhow::Result;
use std::future::Future;
use std::net::Ipv6Addr;
use std::net::SocketAddr;
use wtransport::connection::Connecting;
use wtransport::tls::Certificate;
use wtransport::Endpoint;
use wtransport::ServerConfig;

pub const WEBTRANSPORT_PORT: u16 = 4433;

pub async fn webtransport_server<F, T>(
    certificate: SelfCertificate,
    connection_handler: F,
) -> Result<()>
where
    F: Fn(Connecting) -> T,
    T: Future + Send + 'static,
    T::Output: Send + 'static,
{
    let certificate = Certificate::new(vec![certificate.certificate], certificate.key);

    let config = ServerConfig::builder()
        .with_bind_address(SocketAddr::new(
            Ipv6Addr::UNSPECIFIED.into(),
            WEBTRANSPORT_PORT,
        ))
        .with_certificate(certificate);

    let endpoint = Endpoint::server(config).context("Cannot build WebTransport endpoint")?;

    loop {
        let incoming_connection = endpoint.accept().await.context("Endpoint failed")?;
        tokio::spawn(connection_handler(incoming_connection));
    }
}
