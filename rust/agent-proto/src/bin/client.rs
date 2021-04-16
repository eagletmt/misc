#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    use std::convert::TryFrom as _;

    let socket_path = std::env::var("AGENT_PROTO_SOCK")?;
    let channel = tonic::transport::Endpoint::try_from("http://[::]:50051")?
        .connect_with_connector(tower::service_fn(move |_| {
            tokio::net::UnixStream::connect(socket_path.clone())
        }))
        .await?;
    let mut client = agent_proto::agent_service_client::AgentServiceClient::new(channel);
    let resp = client
        .increment(tonic::Request::new(agent_proto::IncrementRequest { n: 1 }))
        .await?;
    println!("total = {}", resp.into_inner().total);
    Ok(())
}
