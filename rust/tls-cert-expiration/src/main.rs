use chrono::TimeZone as _;
use std::convert::TryFrom as _;
use std::io::Write as _;
use structopt::StructOpt as _;
use x509_parser::traits::FromDer as _;

#[derive(structopt::StructOpt)]
struct Opt {
    #[structopt(short, long, default_value = "443")]
    port: u16,
    host: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();

    let mut tcp_stream = std::net::TcpStream::connect((opt.host.as_str(), opt.port))?;
    let dns_name = rustls::ServerName::try_from(opt.host.as_str())?;
    let mut root_certs = rustls::RootCertStore::empty();
    for cert in rustls_native_certs::load_native_certs()? {
        root_certs.add(&rustls::Certificate(cert.0))?;
    }
    let config = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_certs)
        .with_no_client_auth();
    let mut client = rustls::ClientConnection::new(std::sync::Arc::new(config), dns_name)?;
    while client.wants_write() {
        client.write_tls(&mut tcp_stream)?;
    }
    tcp_stream.flush()?;
    while client.is_handshaking() && client.peer_certificates().is_none() {
        client.read_tls(&mut tcp_stream)?;
        client.process_new_packets()?;
    }
    let certs = client.peer_certificates().unwrap();
    let (_, cert) = x509_parser::certificate::X509Certificate::from_der(certs[0].as_ref())?;
    println!("{}", opt.host);
    println!(
        "    {}",
        chrono::Local
            .timestamp(cert.validity().not_before.timestamp(), 0)
            .to_rfc3339()
    );
    println!(
        "    {}",
        chrono::Local
            .timestamp(cert.validity().not_after.timestamp(), 0)
            .to_rfc3339()
    );
    let duration = cert
        .validity()
        .time_to_expiration()
        .expect("time_to_expiration failed");
    let rc = if duration <= std::time::Duration::from_secs(30 * 24 * 60 * 60) {
        1
    } else {
        0
    };

    std::process::exit(rc);
}
