use anyhow::anyhow;
use anyhow::Result;
use pathsearch::find_executable_in_path;
use std::process::Stdio;
use tokio::process::Child;
use tokio::process::Command;

const GOOGLE_CHROME_EXES: [&str; 3] = ["google-chrome", "google-chrome-stable", "chrome"];

pub struct BrowserHandler(Child);

impl BrowserHandler {
    pub async fn wait(mut self) {
        let _ = self.0.wait().await;
    }
}

/// Launches the browser as child process.
pub fn launch_browser(origin: &str, fingerprint: &str) -> Result<BrowserHandler> {
    let chrome_path = GOOGLE_CHROME_EXES
        .iter()
        .find_map(|command| find_executable_in_path(command))
        .ok_or_else(|| anyhow!("Cannot find Google Chrome"))?;

    let origin_arg = format!("--origin-to-force-quic-on={origin}");
    let skip_arg = format!("--ignore-certificate-errors-spki-list={fingerprint}");

    let child = Command::new(chrome_path)
        .arg(origin_arg)
        .arg(skip_arg)
        .arg("http://localhost:8080")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    Ok(BrowserHandler(child))
}
