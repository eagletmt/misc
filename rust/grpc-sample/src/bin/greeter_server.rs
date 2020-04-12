#[derive(Default)]
struct MyGreeter {}

#[tonic::async_trait]
impl grpc_sample::hello::greeter_server::Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: tonic::Request<grpc_sample::hello::HelloRequest>,
    ) -> Result<tonic::Response<grpc_sample::hello::HelloReply>, tonic::Status> {
        let reply = grpc_sample::hello::HelloReply {
            message: format!("Hello {}!", request.into_inner().name),
        };
        Ok(tonic::Response::new(reply))
    }

    async fn say_hello_slow(
        &self,
        request: tonic::Request<grpc_sample::hello::HelloRequest>,
    ) -> Result<tonic::Response<grpc_sample::hello::HelloReply>, tonic::Status> {
        std::thread::sleep(std::time::Duration::from_secs(10));
        self.say_hello(request).await
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:5000".parse().unwrap();
    let greeter = MyGreeter::default();
    println!("Listening on {} ...", addr);
    tonic::transport::Server::builder()
        .add_service(grpc_sample::hello::greeter_server::GreeterServer::new(
            greeter,
        ))
        .serve(addr)
        .await?;

    Ok(())
}
