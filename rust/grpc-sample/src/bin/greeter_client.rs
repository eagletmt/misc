extern crate env_logger;
extern crate grpc;
extern crate grpc_sample;

fn main() {
    use grpc_sample::hello_grpc::Greeter;

    env_logger::init();

    let client =
        grpc_sample::hello_grpc::GreeterClient::new_plain("localhost", 5000, Default::default())
            .expect("Unable to initialize GreeterClient");
    let mut request = grpc_sample::hello::HelloRequest::new();
    request.set_name("grpc-sample".to_owned());
    let slow = match std::env::var("SLOW") {
        Ok(v) => v == "1",
        Err(_) => false,
    };
    let response = if slow {
        client.say_hello_slow(grpc::RequestOptions::new(), request)
    } else {
        client.say_hello(grpc::RequestOptions::new(), request)
    };
    println!("{:?}", response.wait());
}
