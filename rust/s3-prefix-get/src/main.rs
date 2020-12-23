use bytes::Buf as _;
use futures::StreamExt as _;
use rusoto_s3::S3 as _;
use structopt::StructOpt as _;
use tokio::io::AsyncWriteExt as _;

#[derive(structopt::StructOpt)]
struct Opt {
    #[structopt(short, long)]
    bucket: String,
    #[structopt(short, long)]
    prefix: String,
    #[structopt(short, long)]
    output_dir: std::path::PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();
    let fs_prefix = format!("{}/", opt.prefix.rsplitn(2, '/').last().unwrap());

    let s3_client = rusoto_s3::S3Client::new(Default::default());

    let mut continuation_token = None;
    loop {
        let resp = s3_client
            .list_objects_v2(rusoto_s3::ListObjectsV2Request {
                bucket: opt.bucket.clone(),
                prefix: Some(opt.prefix.clone()),
                continuation_token,
                ..Default::default()
            })
            .await?;
        continuation_token = resp.next_continuation_token;

        if let Some(contents) = resp.contents {
            let mut futures = Vec::new();
            for content in contents {
                if let Some(key) = content.key {
                    let path = opt.output_dir.join(key.strip_prefix(&fs_prefix).unwrap());
                    futures.push(download(&s3_client, &opt.bucket, key, path));
                }
            }
            let mut futures_unordered = futures::stream::iter(futures).buffer_unordered(16);
            while futures_unordered.next().await.is_some() {}
        }

        if continuation_token.is_none() {
            break;
        }
    }

    Ok(())
}

async fn download<S3>(
    s3_client: &S3,
    bucket: &str,
    key: String,
    path: std::path::PathBuf,
) -> Result<(), Box<dyn std::error::Error>>
where
    S3: rusoto_s3::S3,
{
    let resp = s3_client
        .get_object(rusoto_s3::GetObjectRequest {
            bucket: bucket.to_owned(),
            key: key.clone(),
            ..Default::default()
        })
        .await?;
    if let Some(mut body) = resp.body {
        let file = tokio::fs::File::create(&path).await?;
        let mut writer = tokio::io::BufWriter::new(file);
        while let Some(chunk) = body.next().await {
            let mut chunk = chunk?;
            while chunk.has_remaining() {
                writer.write_buf(&mut chunk).await?;
            }
        }
        writer.shutdown().await?;
        println!("s3://{}/{} -> {}", bucket, key, path.display());
    }
    Ok(())
}
