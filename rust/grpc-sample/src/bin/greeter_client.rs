#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client =
        grpc_sample::hello::greeter_client::GreeterClient::connect("http://localhost:5000").await?;
    let request = tonic::Request::new(grpc_sample::hello::HelloRequest {
        name: "grpc-sample".to_owned(),
    });
    let slow = match std::env::var("SLOW") {
        Ok(v) => v == "1",
        Err(_) => false,
    };
    let response = if slow {
        client.say_hello_slow(request).await?
    } else {
        client.say_hello(request).await?
    };
    println!("{:?}", response);

    Ok(())
}
