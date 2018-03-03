extern crate env_logger;
extern crate grpc;
extern crate grpc_sample;
extern crate libc;
extern crate nix;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

struct GreeterService;

impl grpc_sample::hello_grpc::Greeter for GreeterService {
    fn say_hello(
        &self,
        _options: grpc::RequestOptions,
        request: grpc_sample::hello::HelloRequest,
    ) -> grpc::SingleResponse<grpc_sample::hello::HelloReply> {
        let mut reply = grpc_sample::hello::HelloReply::new();
        reply.set_message(format!("Hello {}", request.get_name()));
        grpc::SingleResponse::completed(reply)
    }

    fn say_hello_slow(
        &self,
        options: grpc::RequestOptions,
        request: grpc_sample::hello::HelloRequest,
    ) -> grpc::SingleResponse<grpc_sample::hello::HelloReply> {
        std::thread::sleep(std::time::Duration::from_secs(10));
        self.say_hello(options, request)
    }
}

lazy_static! {
    static ref ON_TERM_FD: std::sync::atomic::AtomicIsize = std::sync::atomic::ATOMIC_ISIZE_INIT;
}

extern "C" fn on_term(_signo: libc::c_int) {
    let fd = ON_TERM_FD.load(std::sync::atomic::Ordering::Relaxed) as libc::c_int;
    let val = 1 as u64;
    unsafe { libc::write(fd, &val as *const u64 as *const libc::c_void, 8) };
}

fn main() {
    env_logger::init();

    let efd = nix::sys::eventfd::eventfd(0, nix::sys::eventfd::EfdFlags::empty())
        .expect("Unable to create eventfd");
    ON_TERM_FD.store(efd as isize, std::sync::atomic::Ordering::Relaxed);

    let mut sigset = nix::sys::signal::SigSet::empty();
    sigset.add(nix::sys::signal::Signal::SIGTERM);
    sigset.add(nix::sys::signal::Signal::SIGINT);
    let sigaction = nix::sys::signal::SigAction::new(
        nix::sys::signal::SigHandler::Handler(on_term),
        nix::sys::signal::SaFlags::SA_RESTART,
        sigset,
    );
    unsafe {
        nix::sys::signal::sigaction(nix::sys::signal::Signal::SIGTERM, &sigaction)
            .expect("Failed to sigaction");
        nix::sys::signal::sigaction(nix::sys::signal::Signal::SIGINT, &sigaction)
            .expect("Failed to sigaction");
    }

    let mut builder = grpc::ServerBuilder::new_plain();
    builder.http.set_port(5000);
    builder.add_service(grpc_sample::hello_grpc::GreeterServer::new_service_def(
        GreeterService,
    ));
    let _server = builder.build().expect("Unable to build gRPC server");

    info!("Waiting...");
    let mut buf = 0 as u64;
    loop {
        let bytes_read = unsafe { libc::read(efd, (&mut buf) as *mut u64 as *mut libc::c_void, 8) };
        if bytes_read == 8 && buf == 1 {
            break;
        } else {
            warn!("bytes_read={} buf={}", bytes_read, buf);
        }
    }
    unsafe { libc::close(efd) };
    info!("Shutting down...");
}
