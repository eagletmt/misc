const BUCKET_NAME: &str = "gyazo.wanko.cc";
const URL_PREFIX: &str = "https://gyazo.wanko.cc";
const REGION: &str = "ap-northeast-1";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let shared_config = aws_config::from_env().region(REGION).load().await;
    let s3 = aws_sdk_s3::Client::new(&shared_config);

    for arg in std::env::args().skip(1) {
        upload(&s3, std::path::Path::new(&arg)).await?;
    }
    Ok(())
}

async fn upload(
    s3_client: &aws_sdk_s3::Client,
    path: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    use md5::Digest as _;
    let image = tokio::fs::read(path).await?;
    let mut hasher = md5::Md5::new();
    hasher.update(&image);
    let digest = format!("{:x}", hasher.finalize());

    let image_key = format!(
        "{}.{}",
        digest,
        path.extension()
            .map(|ext| ext.to_str().unwrap())
            .unwrap_or("")
    );
    let content_type = path.extension().and_then(guess_content_type);
    let html = render_html(&digest, &image_key).into_bytes();
    println!(
        "{} -> {}/{} (https://s3-{}.amazonaws.com/{}/{})",
        path.display(),
        URL_PREFIX,
        digest,
        REGION,
        BUCKET_NAME,
        digest
    );
    let put_image_future = s3_client
        .put_object()
        .bucket(BUCKET_NAME)
        .storage_class(aws_sdk_s3::model::StorageClass::ReducedRedundancy)
        .key(image_key)
        .content_length(image.len() as i64)
        .body(image.into())
        .set_content_type(content_type)
        .send();
    let put_html_future = s3_client
        .put_object()
        .bucket(BUCKET_NAME)
        .storage_class(aws_sdk_s3::model::StorageClass::ReducedRedundancy)
        .key(digest)
        .content_length(html.len() as i64)
        .body(html.into())
        .content_type("text/html")
        .send();
    let (put_image_result, put_html_result) = futures::join!(put_image_future, put_html_future);
    put_image_result?;
    put_html_result?;
    Ok(())
}

fn render_html(digest: &str, key: &str) -> String {
    let link = format!("{}/{}", URL_PREFIX, key);

    let mut buf = String::new();
    buf.push_str("<!DOCTYPE html><html><head><meta charset='utf-8'><title>");
    buf.push_str(digest);
    buf.push_str(
        "</title><meta name='twitter:card' content='photo'><meta name='twitter:title' \
         content='",
    );
    buf.push_str(key);
    buf.push_str("'><meta name='twitter:description' content='");
    buf.push_str(key);
    buf.push_str("'><meta name='twitter:image' content='");
    buf.push_str(&link);
    buf.push_str("'><meta name='twitter:url' content='");
    buf.push_str(&link);
    buf.push_str("'><meta name='og:image' content='");
    buf.push_str(&link);
    buf.push_str("'></head><body><img alt='");
    buf.push_str(key);
    buf.push_str("' src='");
    buf.push_str(&link);
    buf.push_str("'></body></html>");
    buf
}

fn guess_content_type(ext: &std::ffi::OsStr) -> Option<String> {
    if ext == "png" {
        Some("image/png".to_owned())
    } else if ext == "jpg" {
        Some("image/jpeg".to_owned())
    } else {
        None
    }
}
