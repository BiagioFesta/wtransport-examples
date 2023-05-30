use crate::certificate::SelfCertificate;
use anyhow::Result;
use rustls::ServerConfig as TlsServerConfig;
use std::future::Future;
use std::net::Ipv6Addr;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio_rustls::server::TlsStream;
use tokio_rustls::TlsAcceptor;
use tokio_tungstenite::WebSocketStream;

pub const WEBSOCKET_PORT: u16 = 4434;

pub type WebSocket = WebSocketStream<TlsStream<TcpStream>>;

pub async fn websocket_server<F, T>(
    certificate: SelfCertificate,
    connection_handler: F,
) -> Result<()>
where
    F: Fn(WebSocket) -> T + Send + Copy + 'static,
    T: Future + Send + 'static,
    T::Output: Send + 'static,
{
    let tls_config = Arc::new(
        TlsServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(
                vec![rustls::Certificate(certificate.certificate)],
                rustls::PrivateKey(certificate.key),
            )?,
    );

    let tcp_listener = TcpListener::bind(SocketAddr::new(
        Ipv6Addr::UNSPECIFIED.into(),
        WEBSOCKET_PORT,
    ))
    .await?;

    loop {
        let (tcp_stream, _) = tcp_listener.accept().await?;

        tokio::spawn({
            let tls_config = tls_config.clone();

            async move {
                let tls_acceptor = TlsAcceptor::from(tls_config);
                let tls_stream = tls_acceptor.accept(tcp_stream).await.unwrap();
                let websocket = tokio_tungstenite::accept_async(tls_stream).await.unwrap();
                tokio::spawn(connection_handler(websocket));
            }
        });
    }
}
