use chrono::TimeZone as _;
use clap::Parser as _;
use std::convert::TryFrom as _;
use std::io::Write as _;
use x509_parser::prelude::FromDer as _;

#[derive(Debug, clap::Parser)]
struct Opt {
    #[clap(short, long, default_value = "443")]
    port: u16,
    host: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::parse();

    let mut tcp_stream = std::net::TcpStream::connect((opt.host.as_str(), opt.port))?;
    let dns_name = rustls::pki_types::ServerName::try_from(opt.host.as_str())?.to_owned();
    let mut root_certs = rustls::RootCertStore::empty();
    for cert in rustls_native_certs::load_native_certs().expect("failed to load native certs") {
        root_certs.add(cert)?;
    }
    let config = rustls::ClientConfig::builder()
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
            .timestamp_opt(cert.validity().not_before.timestamp(), 0)
            .unwrap()
            .to_rfc3339()
    );
    println!(
        "    {}",
        chrono::Local
            .timestamp_opt(cert.validity().not_after.timestamp(), 0)
            .unwrap()
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
