extern crate chrono;
extern crate getopts;
extern crate openssl;

fn main() {
    use std::io::Write;

    let mut args = std::env::args();
    let program = args.next().unwrap();

    let mut options = getopts::Options::new();
    options.optopt("p", "port", "Port number (default: 443)", "PORT");
    let mut matches = match options.parse(args) {
        Ok(m) => m,
        Err(msg) => {
            writeln!(std::io::stderr(), "{}", msg).unwrap();
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

    let threshold = chrono::Local::now() + chrono::Duration::days(30);

    let connector = openssl::ssl::SslConnectorBuilder::new(openssl::ssl::SslMethod::tls())
        .expect("SslConnectorBuilder::new failed")
        .build();
    let tcp_stream = std::net::TcpStream::connect((&*host, port))
        .expect("TcpStream::connect failed");
    let tls_stream = connector.connect(&*host, tcp_stream).expect("SslConnector::connect failed");
    let cert = tls_stream.ssl().peer_certificate().expect("No peer_certificate");
    let not_after = from_asn1_time(cert.not_after()).expect("Unable to parse not_after");
    let not_before = from_asn1_time(cert.not_before()).expect("Unable to parse not_before");
    println!("{}", host);
    println!("    {}", not_before);
    println!("    {}", not_after);
    let rc = if not_after <= threshold { 1 } else { 0 };

    std::process::exit(rc);
}

fn print_usage(program: &str, options: &getopts::Options) {
    println!("{}", options.short_usage(&program));
    println!("{}", options.usage("Check TLS certificate expiration"));
}


fn from_asn1_time(t: &openssl::asn1::Asn1TimeRef)
                  -> Result<chrono::DateTime<chrono::Local>, chrono::ParseError> {
    chrono::DateTime::parse_from_str(&t.to_string().replace(" GMT", " +00:00"), "%b %e %T %Y %z")
        .map(|in_utc| in_utc.with_timezone(&chrono::Local))
}
