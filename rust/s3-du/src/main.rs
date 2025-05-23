use clap::Parser as _;

#[derive(Debug, clap::Parser)]
struct Opt {
    #[clap(short, long)]
    bucket: String,
    #[clap(short, long)]
    prefix: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::parse();

    let shared_config = aws_config::load_defaults(aws_config::BehaviorVersion::v2025_01_17()).await;
    let s3_client = aws_sdk_s3::Client::new(&shared_config);

    let mut continuation_token = None;
    let mut size = 0;
    loop {
        let resp = s3_client
            .list_objects_v2()
            .bucket(&opt.bucket)
            .prefix(&opt.prefix)
            .set_continuation_token(continuation_token)
            .send()
            .await?;
        continuation_token = resp.next_continuation_token;

        if let Some(contents) = resp.contents {
            for content in contents {
                size += content.size.expect("content size is none");
            }
        }

        if continuation_token.is_none() {
            break;
        }
    }
    println!("{}", size);

    Ok(())
}
