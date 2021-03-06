const BUCKET_NAME: &str = "gyazo.wanko.cc";
const URL_PREFIX: &str = "https://gyazo.wanko.cc";
const REGION: rusoto_core::Region = rusoto_core::Region::ApNortheast1;

#[tokio::main]
async fn main() {
    let s3 = rusoto_s3::S3Client::new(REGION);

    for arg in std::env::args().skip(1) {
        upload(&s3, std::path::Path::new(&arg)).await;
    }
}

async fn upload<S3>(s3: &S3, path: &std::path::Path)
where
    S3: rusoto_s3::S3,
{
    let mut file = std::fs::File::open(path).expect("std::fs::File::open");
    let mut image = Vec::new();
    use std::io::Read;
    file.read_to_end(&mut image).expect("read_to_end");
    let mut md5 = crypto::md5::Md5::new();
    use crypto::digest::Digest;
    md5.input(&image);
    let digest = md5.result_str();

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
        REGION.name(),
        BUCKET_NAME,
        digest
    );
    let put_image_future = s3.put_object(rusoto_s3::PutObjectRequest {
        bucket: BUCKET_NAME.to_owned(),
        acl: Some("public-read".to_owned()),
        storage_class: Some("REDUCED_REDUNDANCY".to_owned()),
        key: image_key,
        content_length: Some(image.len() as i64),
        body: Some(image.into()),
        content_type,
        ..Default::default()
    });
    let put_html_future = s3.put_object(rusoto_s3::PutObjectRequest {
        bucket: BUCKET_NAME.to_owned(),
        acl: Some("public-read".to_owned()),
        storage_class: Some("REDUCED_REDUNDANCY".to_owned()),
        key: digest,
        content_length: Some(html.len() as i64),
        body: Some(html.into()),
        content_type: Some("text/html".to_owned()),
        ..Default::default()
    });
    let (put_image_result, put_html_result) = futures::join!(put_image_future, put_html_future);
    put_image_result.expect("s3.put_object (html)");
    put_html_result.expect("s3.put_object (image)");
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
