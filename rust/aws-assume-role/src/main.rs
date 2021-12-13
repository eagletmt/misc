#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = clap::App::new(clap::crate_name!())
        .version(clap::crate_version!())
        .arg(
            clap::Arg::with_name("role-arn")
                .short("r")
                .long("role-arn")
                .value_name("ROLE_ARN")
                .takes_value(true)
                .required(true),
        )
        .get_matches();
    let role_arn = matches.value_of("role-arn").unwrap().to_owned();
    let role_session_name = format!(
        "{}-{}",
        std::env::var("USER")?,
        std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)?
            .as_secs()
    );

    let shared_config = aws_config::load_from_env().await;
    let sts_client = aws_sdk_sts::Client::new(&shared_config);
    let resp = sts_client
        .assume_role()
        .role_arn(role_arn)
        .role_session_name(role_session_name)
        .send()
        .await?;
    let creds = resp.credentials.unwrap();
    println!(
        "AWS_ACCESS_KEY_ID={}\nAWS_SECRET_ACCESS_KEY={}\nAWS_SESSION_TOKEN={}",
        creds.access_key_id.expect("access_key_id is missing"),
        creds
            .secret_access_key
            .expect("secret_access_key is missing"),
        creds.session_token.expect("session_token is missing"),
    );

    Ok(())
}
