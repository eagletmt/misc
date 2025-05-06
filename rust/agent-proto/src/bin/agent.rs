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
    tracing_subscriber::fmt::init();
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
        let stdin = std::fs::File::open("/dev/null")?;
        nix::unistd::dup2_stdin(stdin)?;
        let stdout =
            std::fs::File::create(socket_dir.path().join(format!("stdout.{}.log", child_pid)))?;
        let stderr =
            std::fs::File::create(socket_dir.path().join(format!("stderr.{}.log", child_pid)))?;
        nix::unistd::dup2_stdout(stdout)?;
        nix::unistd::dup2_stderr(stderr)?;
    }

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let uds = tokio::net::UnixListener::bind(&socket_path)?;
        let uds_stream = tokio_stream::wrappers::UnixListenerStream::new(uds);

        let agent = Agent {
            total: std::sync::Arc::new(std::sync::Mutex::new(0)),
        };
        tracing::info!(
            "Starting server {} (pid: {})",
            socket_path.display(),
            child_pid,
        );
        tonic::transport::Server::builder()
            .add_service(agent_proto::agent_service_server::AgentServiceServer::new(
                agent,
            ))
            .serve_with_incoming_shutdown(uds_stream, shutdown())
            .await?;
        tracing::info!("Exiting");
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
    tracing::info!("Got {}", sig);
}
