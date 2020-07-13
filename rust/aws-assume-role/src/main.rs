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

    use rusoto_sts::Sts;

    let sts_client = rusoto_sts::StsClient::new(Default::default());
    let resp = sts_client
        .assume_role(rusoto_sts::AssumeRoleRequest {
            role_arn,
            role_session_name,
            ..Default::default()
        })
        .await?;
    let creds = resp.credentials.unwrap();
    println!(
        "AWS_ACCESS_KEY_ID={}\nAWS_SECRET_ACCESS_KEY={}\nAWS_SESSION_TOKEN={}",
        creds.access_key_id, creds.secret_access_key, creds.session_token
    );

    Ok(())
}
