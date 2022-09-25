use tokio::io::AsyncBufReadExt as _;

pub async fn launch() -> anyhow::Result<(
    tokio::process::Child,
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
)> {
    let user_data_dir = tempfile::TempDir::new()?;
    const DEFAULT_ARGS: &[&str] = &[
      "--allow-pre-commit-input",
      "--disable-background-networking",
      "--enable-features=NetworkServiceInProcess2",
      "--disable-background-timer-throttling",
      "--disable-backgrounding-occluded-windows",
      "--disable-breakpad",
      "--disable-client-side-phishing-detection",
      "--disable-component-extensions-with-background-pages",
      "--disable-default-apps",
      "--disable-dev-shm-usage",
      "--disable-extensions",
      "--disable-features=Translate,BackForwardCache,AcceptCHFrame,AvoidUnnecessaryBeforeUnloadCheckSync",
      "--disable-hang-monitor",
      "--disable-ipc-flooding-protection",
      "--disable-popup-blocking",
      "--disable-prompt-on-repost",
      "--disable-renderer-backgrounding",
      "--disable-sync",
      "--force-color-profile=srgb",
      "--metrics-recording-only",
      "--no-first-run",
      "--enable-automation",
      "--password-store=basic",
      "--use-mock-keychain",
      "--enable-blink-features=IdleDetection",
      "--export-tagged-pdf",
    ];
    const HEADLESS_ARGS: &[&str] = &["--headless", "--hide-scrollbars", "--mute-audio"];
    let mut child = tokio::process::Command::new("chromium")
        .stderr(std::process::Stdio::piped())
        .args(DEFAULT_ARGS)
        .args(HEADLESS_ARGS)
        .arg("--remote-debugging-port=0")
        .arg(format!(
            "--user-data-dir={}",
            user_data_dir.path().display()
        ))
        .spawn()?;
    let stderr = child
        .stderr
        .take()
        .expect("child process doesn't have stderr");
    let mut reader = tokio::io::BufReader::new(stderr).lines();
    while let Some(line) = reader.next_line().await? {
        const DEVTOOLS_PREFIX: &str = "DevTools listening on ";
        if let Some(endpoint) = line.strip_prefix(DEVTOOLS_PREFIX) {
            let (ws, _) = tokio_tungstenite::connect_async(endpoint).await?;
            return Ok((child, ws));
        }
    }
    anyhow::bail!("failed to find DevTools endpoint");
}
