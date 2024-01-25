use clap::Parser as _;
use futures::StreamExt as _;
use tokio::io::AsyncWriteExt as _;

#[derive(clap::Parser)]
struct Opt {
    #[clap(short, long)]
    bucket: String,
    #[clap(short, long)]
    prefix: String,
    #[clap(short, long)]
    output_dir: std::path::PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::parse();

    let shared_config = aws_config::load_defaults(aws_config::BehaviorVersion::v2023_11_09()).await;
    let s3_client = aws_sdk_s3::Client::new(&shared_config);

    let mut continuation_token = None;
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
            let mut futures = Vec::new();
            for content in contents {
                if let Some(key) = content.key {
                    let path = opt.output_dir.join(&key);
                    futures.push(download(&s3_client, &opt.bucket, key, path));
                }
            }
            let mut futures_unordered = futures::stream::iter(futures).buffer_unordered(16);
            while futures_unordered.next().await.transpose()?.is_some() {}
        }

        if continuation_token.is_none() {
            break;
        }
    }

    Ok(())
}

async fn download(
    s3_client: &aws_sdk_s3::Client,
    bucket: &str,
    key: String,
    path: std::path::PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let resp = s3_client
        .get_object()
        .bucket(bucket)
        .key(&key)
        .send()
        .await?;
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let file = tokio::fs::File::create(&path).await?;
    let mut writer = tokio::io::BufWriter::new(file);
    let mut reader = resp.body.into_async_read();
    tokio::io::copy(&mut reader, &mut writer).await?;
    writer.shutdown().await?;
    println!("s3://{}/{} -> {}", bucket, key, path.display());
    Ok(())
}
