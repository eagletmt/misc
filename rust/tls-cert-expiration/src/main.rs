extern crate openssl;
extern crate chrono;

fn main() {
    let threshold = chrono::Local::now() + chrono::Duration::days(30);

    let mut rc = 0;
    for arg in std::env::args().skip(1) {
        let connector = openssl::ssl::SslConnectorBuilder::new(openssl::ssl::SslMethod::tls())
            .expect("SslConnectorBuilder::new failed")
            .build();
        let tcp_stream = std::net::TcpStream::connect((&*arg, 443))
            .expect("TcpStream::connect failed");
        let tls_stream = connector.connect(&arg, tcp_stream).expect("SslConnector::connect failed");
        let cert = tls_stream.ssl().peer_certificate().expect("No peer_certificate");
        let not_after = from_asn1_time(cert.not_after()).expect("Unable to parse not_after");
        let not_before = from_asn1_time(cert.not_before()).expect("Unable to parse not_before");
        println!("{}", arg);
        println!("    {}", not_before);
        println!("    {}", not_after);
        if not_after <= threshold {
            rc += 1;
        }
    }

    std::process::exit(rc);
}

fn from_asn1_time(t: &openssl::asn1::Asn1TimeRef)
                  -> Result<chrono::DateTime<chrono::Local>, chrono::ParseError> {
    chrono::DateTime::parse_from_str(&t.to_string().replace(" GMT", " +00:00"), "%b %e %T %Y %z")
        .map(|in_utc| in_utc.with_timezone(&chrono::Local))
}
