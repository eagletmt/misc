#[derive(Debug, clap::Parser)]
struct Args {
    #[clap(short, long)]
    role_arn: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use clap::Parser as _;
    let args = Args::parse();

    let role_session_name = format!(
        "{}-{}",
        std::env::var("USER")?,
        std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)?
            .as_secs()
    );

    let shared_config = aws_config::load_defaults(aws_config::BehaviorVersion::v2023_11_09()).await;
    let sts_client = aws_sdk_sts::Client::new(&shared_config);
    let resp = sts_client
        .assume_role()
        .role_arn(args.role_arn)
        .role_session_name(role_session_name)
        .send()
        .await?;
    let creds = resp.credentials.unwrap();
    println!(
        "AWS_ACCESS_KEY_ID={}\nAWS_SECRET_ACCESS_KEY={}\nAWS_SESSION_TOKEN={}",
        creds.access_key_id, creds.secret_access_key, creds.session_token,
    );

    Ok(())
}
