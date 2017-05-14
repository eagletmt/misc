extern crate crypto;
extern crate rusoto;

const BUCKET_NAME: &'static str = "gyazo.wanko.cc";
const URL_PREFIX: &'static str = "https://gyazo.wanko.cc";
const REGION: rusoto::Region = rusoto::Region::ApNortheast1;

fn main() {

    let s3 = rusoto::s3::S3Client::new(rusoto::default_tls_client().expect("default_tls_client"),
                                       rusoto::DefaultCredentialsProvider::new()
                                           .expect("DefaultCredentialsProvider::new"),
                                       REGION);

    for arg in std::env::args().skip(1) {
        upload(&s3, std::path::Path::new(&arg));
    }
}

fn upload<P, D>(s3: &rusoto::s3::S3Client<P, D>, path: &std::path::Path)
    where P: rusoto::ProvideAwsCredentials,
          D: rusoto::DispatchSignedRequest
{
    if std::env::var("IGNORE_TWITTER_CHECK")
           .map(|check| check != "1")
           .unwrap_or(true) {
        let size = std::fs::metadata(path).expect("std::fs::metadata").len();
        if size >= (1 << 20) {
            println!("{} is larger than 1MB", path.display());
            return;
        }
    }

    let mut file = std::fs::File::open(path).expect("std::fs::File::open");
    let mut image = Vec::new();
    use std::io::Read;
    file.read_to_end(&mut image).expect("read_to_end");
    let mut md5 = crypto::md5::Md5::new();
    use crypto::digest::Digest;
    md5.input(&image);
    let digest = md5.result_str();

    let image_key = format!("{}.{}",
                            digest,
                            path.extension()
                                .map(|ext| ext.to_str().unwrap())
                                .unwrap_or(""));
    let content_type = path.extension().and_then(guess_content_type);
    let html = render_html(&digest, &image_key);
    println!("{} -> {}/{} (https://s3-{}.amazonaws.com/{}/{})",
             path.display(),
             URL_PREFIX,
             digest,
             REGION,
             BUCKET_NAME,
             digest);
    s3.put_object(&rusoto::s3::PutObjectRequest {
                        bucket: BUCKET_NAME.to_owned(),
                        acl: Some("public-read".to_owned()),
                        storage_class: Some("REDUCED_REDUNDANCY".to_owned()),
                        key: image_key,
                        body: Some(image),
                        content_type: content_type,
                        ..rusoto::s3::PutObjectRequest::default()
                    })
        .expect("s3.put_object (image)");
    s3.put_object(&rusoto::s3::PutObjectRequest {
                        bucket: BUCKET_NAME.to_owned(),
                        acl: Some("public-read".to_owned()),
                        storage_class: Some("REDUCED_REDUNDANCY".to_owned()),
                        key: digest,
                        body: Some(html.into_bytes()),
                        content_type: Some("text/html".to_owned()),
                        ..rusoto::s3::PutObjectRequest::default()
                    })
        .expect("s3.put_object (html)");
}

fn render_html(digest: &str, key: &str) -> String {
    let link = format!("{}/{}", URL_PREFIX, key);

    let mut buf = String::new();
    buf.push_str("<!DOCTYPE html><html><head><meta charset='utf-8'><title>");
    buf.push_str(digest);
    buf.push_str("</title><meta name='twitter:card' content='photo'><meta name='twitter:title' \
                  content='");
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
    } else {
        None
    }
}
