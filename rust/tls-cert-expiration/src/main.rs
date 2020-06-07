fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args();
    let program = args.next().unwrap();

    let mut options = getopts::Options::new();
    options.optopt("p", "port", "Port number (default: 443)", "PORT");
    let mut matches = match options.parse(args) {
        Ok(m) => m,
        Err(msg) => {
            eprintln!("{}", msg);
            print_usage(&program, &options);
            std::process::exit(1);
        }
    };

    let host = if matches.free.is_empty() {
        print_usage(&program, &options);
        std::process::exit(2);
    } else {
        matches.free.remove(0)
    };
    let port = matches.opt_str("p").map_or(443, |v| v.parse().unwrap());

    let mut tcp_stream = std::net::TcpStream::connect((&*host, port))?;
    let dns_name = webpki::DNSNameRef::try_from_ascii_str(&host)?;
    let mut config = rustls::ClientConfig::new();
    config.root_store =
        rustls_native_certs::load_native_certs().expect("Failed to load local certificates");
    let mut client_session = rustls::ClientSession::new(&std::sync::Arc::new(config), dns_name);
    use rustls::Session;
    while client_session.wants_write() {
        client_session.write_tls(&mut tcp_stream)?;
    }
    while client_session.wants_read() && client_session.get_peer_certificates().is_none() {
        client_session.read_tls(&mut tcp_stream)?;
        client_session.process_new_packets()?;
    }
    let certs = client_session.get_peer_certificates().unwrap();
    let (_, cert) =
        x509_parser::parse_x509_der(certs[0].as_ref()).expect("Failed to parse peer certificate");
    println!("{}", host);
    println!("    {}", cert.tbs_certificate.validity.not_before.rfc3339());
    println!("    {}", cert.tbs_certificate.validity.not_after.rfc3339());
    let duration = cert
        .tbs_certificate
        .validity
        .time_to_expiration()
        .expect("time_to_expiration failed");
    let rc = if duration <= std::time::Duration::from_secs(30 * 24 * 60 * 60) {
        1
    } else {
        0
    };

    std::process::exit(rc);
}

fn print_usage(program: &str, options: &getopts::Options) {
    println!("{}", options.short_usage(program));
    println!("{}", options.usage("Check TLS certificate expiration"));
}
