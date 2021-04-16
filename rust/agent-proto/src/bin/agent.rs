struct Agent {
    total: std::sync::Arc<std::sync::Mutex<u64>>,
}

#[tonic::async_trait]
impl agent_proto::agent_service_server::AgentService for Agent {
    async fn increment(
        &self,
        request: tonic::Request<agent_proto::IncrementRequest>,
    ) -> Result<tonic::Response<agent_proto::IncrementResponse>, tonic::Status> {
        let mut n = self
            .total
            .lock()
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        *n += request.into_inner().n;
        Ok(tonic::Response::new(agent_proto::IncrementResponse {
            total: *n,
        }))
    }
}

// Do not use tokio::main because fork(2) inside Tokio runtime behaves badly
fn main() -> Result<(), anyhow::Error> {
    env_logger::init();
    nix::sys::stat::umask(nix::sys::stat::Mode::from_bits(0o077).unwrap());

    let socket_dir = tempfile::Builder::new().prefix("agent-proto-").tempdir()?;
    let parent_pid = std::process::id();
    let socket_path = socket_dir.path().join(format!("agent.{}.sock", parent_pid));

    if let nix::unistd::ForkResult::Parent { child } = unsafe { nix::unistd::fork() }? {
        println!(
            "AGENT_PROTO_SOCK={}; export AGENT_PROTO_SOCK;",
            socket_path.display()
        );
        println!("AGENT_PROTO_PID={}; export AGENT_PROTO_PID;", child);
        std::process::exit(0);
    }

    unsafe {
        libc::prctl(libc::PR_SET_DUMPABLE, 0);
        libc::setrlimit(
            libc::RLIMIT_CORE,
            &libc::rlimit {
                rlim_cur: 0,
                rlim_max: 0,
            },
        );
    }
    nix::unistd::setsid()?;
    std::env::set_current_dir("/")?;
    let child_pid = std::process::id();
    {
        use std::os::unix::io::AsRawFd as _;
        let stdin = std::fs::File::open("/dev/null")?;
        nix::unistd::dup2(stdin.as_raw_fd(), std::io::stdin().as_raw_fd())?;
        let stdout =
            std::fs::File::create(socket_dir.path().join(format!("stdout.{}.log", child_pid)))?;
        let stderr =
            std::fs::File::create(socket_dir.path().join(format!("stderr.{}.log", child_pid)))?;
        nix::unistd::dup2(stdout.as_raw_fd(), std::io::stdout().as_raw_fd())?;
        nix::unistd::dup2(stderr.as_raw_fd(), std::io::stderr().as_raw_fd())?;
    }

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        use futures::TryFutureExt as _;

        let incoming = {
            let uds = tokio::net::UnixListener::bind(&socket_path)?;
            async_stream::stream! {
                while let item = uds.accept().map_ok(|(st, _)| UnixStream(st)).await {
                    yield item;
                }
            }
        };

        let agent = Agent {
            total: std::sync::Arc::new(std::sync::Mutex::new(0)),
        };
        log::info!(
            "Starting server {} (pid: {})",
            socket_path.display(),
            child_pid,
        );
        tonic::transport::Server::builder()
            .add_service(agent_proto::agent_service_server::AgentServiceServer::new(
                agent,
            ))
            .serve_with_incoming_shutdown(incoming, shutdown())
            .await?;
        log::info!("Exiting");
        Ok(())
    })
}

async fn shutdown() {
    let mut sigint = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())
        .expect("Failed to set signal handler for SIGINT");
    let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .expect("Failed to set signal handler for SIGTERM");
    let sig = tokio::select! {
        _ = sigint.recv() => "SIGINT",
        _ = sigterm.recv() => "SIGTERM",
    };
    log::info!("Got {}", sig);
}

#[derive(Debug)]
struct UnixStream(tokio::net::UnixStream);

impl tonic::transport::server::Connected for UnixStream {}

impl tokio::io::AsyncRead for UnixStream {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.0).poll_read(cx, buf)
    }
}

impl tokio::io::AsyncWrite for UnixStream {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        std::pin::Pin::new(&mut self.0).poll_write(cx, buf)
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.0).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.0).poll_shutdown(cx)
    }
}
