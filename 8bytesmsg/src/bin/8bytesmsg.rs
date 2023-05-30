use anyhow::Result;
use futures_util::SinkExt;
use std::time::Duration;
use tokio_tungstenite::tungstenite::Message;
use wtransport::connection::Connecting;
use wtransport_examples_utilities::browser::launch_browser;
use wtransport_examples_utilities::certificate::generate_certificate;
use wtransport_examples_utilities::http_server::http_server;
use wtransport_examples_utilities::websocket_server::websocket_server;
use wtransport_examples_utilities::websocket_server::WebSocket;
use wtransport_examples_utilities::webtransport_server::webtransport_server;

const MAX_VALUE: u64 = 1_000_000;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let certificate = generate_certificate("localhost")?;
    println!("Certificate fingerprint: {}", certificate.fingerprint);

    let http_server = http_server("8bytesmsg/http", &certificate.fingerprint);
    let webtransport_server = webtransport_server(certificate.clone(), webtransport);
    let websocket_server = websocket_server(certificate.clone(), websocket);
    let browser = launch_browser("localhost:4433", &certificate.fingerprint)?;

    tokio::select! {
        result = http_server => {
            result?;
        }
        result = webtransport_server => {
            result?;
        }
        result = websocket_server => {
            result?;
        }
        _ = browser.wait() => {}
    }

    Ok(())
}

async fn webtransport(connecting: Connecting) {
    let connection = connecting.await.unwrap();

    for i in 0..MAX_VALUE {
        connection.send_datagram(i.to_be_bytes()).unwrap();
        tokio::task::yield_now().await;
    }

    tokio::time::sleep(Duration::from_secs(3)).await;
}

async fn websocket(mut websocket: WebSocket) {
    for i in 0..MAX_VALUE {
        websocket
            .send(Message::Binary(i.to_be_bytes().to_vec()))
            .await
            .unwrap();
    }
}
