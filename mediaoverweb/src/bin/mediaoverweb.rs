use anyhow::Result;
use frame_generator::FrameGenerator;
use futures_util::SinkExt;
use std::time::Duration;
use tokio_tungstenite::tungstenite::Message;
use wtransport::connection::{Connecting};
use wtransport_examples_utilities::browser::launch_browser;
use wtransport_examples_utilities::certificate::generate_certificate;
use wtransport_examples_utilities::http_server::http_server;
use wtransport_examples_utilities::websocket_server::{websocket_server, WebSocket};
use wtransport_examples_utilities::webtransport_server::webtransport_server;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let certificate = generate_certificate("localhost")?;
    println!("Certificate fingerprint: {}", certificate.fingerprint);

    let http_server = http_server("mediaoverweb/http", &certificate.fingerprint);
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
    let mut frame_generator = FrameGenerator::new(800, 600);

    loop {
        let frame = frame_generator.next_frame();

        connection
            .open_uni()
            .await
            .unwrap()
            .write_all(frame.data())
            .await
            .unwrap();

        tokio::time::sleep(Duration::from_millis(16)).await;
    }
}

async fn websocket(mut websocket: WebSocket) {
    let mut frame_generator = FrameGenerator::new(800, 600);

    loop {
        let frame = frame_generator.next_frame();

        websocket
            .send(Message::Binary(frame.to_vec()))
            .await
            .unwrap();
    }
}

mod frame_generator {
    use openh264::encoder::EncodedBitStream;
    use openh264::encoder::Encoder;
    use openh264::encoder::EncoderConfig;
    use openh264::formats::YUVBuffer;
    use std::f32::consts::PI;
    use std::time::Instant;
    use tiny_skia::Paint;
    use tiny_skia::Pixmap;
    use tiny_skia::Rect;
    use tiny_skia::Transform;

    pub struct Frame {
        data: Box<[u8]>,
    }

    impl Frame {
        pub fn data(&self) -> &[u8] {
            &self.data
        }

        pub fn to_vec(self) -> Vec<u8> {
            self.data.to_vec()
        }

        fn with_bitstream(bitstream: EncodedBitStream) -> Self {
            let data = bitstream.to_vec().into_boxed_slice();
            Frame { data }
        }
    }

    pub struct FrameGenerator {
        time: Instant,
        encoder: Encoder,
        width: u32,
        height: u32,
    }

    impl FrameGenerator {
        pub fn new(width: u32, height: u32) -> Self {
            let time = Instant::now();
            let encoder = Encoder::with_config(EncoderConfig::new(width, height)).unwrap();

            Self {
                encoder,
                time,
                width,
                height,
            }
        }

        pub fn next_frame(&mut self) -> Frame {
            let rgb = self.draw_frame();
            let yuv_buffer = YUVBuffer::with_rgb(self.width as usize, self.height as usize, &rgb);

            let bitstream = self.encoder.encode(&yuv_buffer).unwrap();
            Frame::with_bitstream(bitstream)
        }

        fn draw_frame(&mut self) -> Box<[u8]> {
            const PERIOD: f32 = 10.0;

            let delta = self.time.elapsed();

            let size = std::cmp::min(self.width, self.height) as f32
                * (delta.as_secs_f32() * PI / PERIOD).sin().abs();

            let rect = Rect::from_xywh(0.0, 0.0, size, size).unwrap();

            let mut paint = Paint::default();
            paint.set_color_rgba8(255, 0, 0, 255);

            let mut pixmap = Pixmap::new(self.width, self.height).unwrap();
            pixmap.fill_rect(rect, &paint, Transform::identity(), None);

            let rgb = pixmap
                .take()
                .into_iter()
                .enumerate()
                .filter_map(|(index, pixel)| (((index + 1) % 4) != 0).then(|| pixel))
                .collect();

            rgb
        }
    }
}
