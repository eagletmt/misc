#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut resolver = trust_dns_resolver::AsyncResolver::tokio_from_system_conf()?;

    for arg in std::env::args().skip(1) {
        resolve_name(&mut resolver, arg).await?;
    }
    Ok(())
}

async fn resolve_name(
    resolver: &mut trust_dns_resolver::TokioAsyncResolver,
    name: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let type_style = ansi_term::Style::new().fg(ansi_term::Color::Yellow);
    let name_style = ansi_term::Style::new().fg(ansi_term::Color::Green);
    let addr_style = ansi_term::Style::new().fg(ansi_term::Color::Blue);

    let mx_handle = tokio::spawn(resolve_mx(
        resolver.clone(),
        name.clone(),
        type_style,
        name_style,
    ));
    let txt_handle = tokio::spawn(resolve_txt(resolver.clone(), name.clone(), type_style));
    let caa_handle = tokio::spawn(resolve_caa(
        resolver.clone(),
        name.clone(),
        type_style,
        name_style,
    ));

    mx_handle.await?;
    txt_handle.await?;
    caa_handle.await?;

    let mut name = name;
    loop {
        let mut resolved = false;
        if let Ok(resp) = resolver
            .lookup(
                name.as_str(),
                trust_dns_resolver::proto::rr::RecordType::CNAME,
            )
            .await
        {
            for cname in resp {
                resolved = true;
                let next_name = cname.as_cname().unwrap().to_string();
                println!(
                    "{} {} {}",
                    name,
                    type_style.paint("CNAME"),
                    name_style.paint(&next_name)
                );
                name = next_name;
            }
        }
        if !resolved {
            break;
        }
    }
    let name = name;

    let mut addrs = Vec::new();
    if let Ok(resp) = resolver.ipv4_lookup(name.as_str()).await {
        for a in resp {
            println!(
                "{} {} {}",
                name,
                type_style.paint("A"),
                addr_style.paint(a.to_string())
            );
            addrs.push(std::net::IpAddr::from(a));
        }
    }

    if let Ok(resp) = resolver.ipv6_lookup(name.as_str()).await {
        for aaaa in resp {
            println!(
                "{} {} {}",
                name,
                type_style.paint("AAAA"),
                addr_style.paint(aaaa.to_string())
            );
            addrs.push(std::net::IpAddr::from(aaaa));
        }
    }

    let mut addr_handles = Vec::new();
    for addr in addrs {
        addr_handles.push(tokio::spawn(resolve_ptr(
            resolver.clone(),
            addr,
            type_style,
            name_style,
        )));
    }
    for handle in addr_handles {
        handle.await?;
    }
    Ok(())
}

async fn resolve_mx(
    resolver: trust_dns_resolver::TokioAsyncResolver,
    name: String,
    type_style: ansi_term::Style,
    name_style: ansi_term::Style,
) {
    if let Ok(resp) = resolver.mx_lookup(name.as_str()).await {
        let mut records: Vec<_> = resp.into_iter().collect();
        records.sort_unstable_by(|x, y| {
            x.preference()
                .cmp(&y.preference())
                .then_with(|| x.exchange().cmp(y.exchange()))
        });
        for mx in records {
            println!(
                "{} {} {} {}",
                name,
                type_style.paint("MX"),
                name_style.paint(mx.exchange().to_utf8()),
                mx.preference()
            );
        }
    }
}

async fn resolve_txt(
    resolver: trust_dns_resolver::TokioAsyncResolver,
    name: String,
    type_style: ansi_term::Style,
) {
    if let Ok(resp) = resolver.txt_lookup(name.as_str()).await {
        for txt in resp {
            for data in txt.txt_data() {
                println!(
                    "{} {} {}",
                    name,
                    type_style.paint("TXT"),
                    String::from_utf8_lossy(data),
                );
            }
        }
    }
}

async fn resolve_caa(
    resolver: trust_dns_resolver::TokioAsyncResolver,
    name: String,
    type_style: ansi_term::Style,
    name_style: ansi_term::Style,
) {
    if let Ok(resp) = resolver
        .lookup(
            name.as_str(),
            trust_dns_resolver::proto::rr::RecordType::CAA,
        )
        .await
    {
        for rdata in resp {
            let caa = rdata.as_caa().unwrap();
            use trust_dns_resolver::proto::rr::rdata::caa::{Property, Value};
            match (caa.tag(), caa.value()) {
                (Property::Issue, Value::Issuer(Some(domain), _)) => {
                    println!(
                        "{} {} issue {} (critical={})",
                        name,
                        type_style.paint("CAA"),
                        name_style.paint(domain.to_utf8()),
                        caa.issuer_critical()
                    );
                }
                (Property::IssueWild, Value::Issuer(Some(domain), _)) => {
                    println!(
                        "{} {} issuewild {} (critical={})",
                        name,
                        type_style.paint("CAA"),
                        name_style.paint(domain.to_utf8()),
                        caa.issuer_critical()
                    );
                }
                (Property::Iodef, Value::Url(url)) => {
                    println!(
                        "{} {} iodef {} (critical={})",
                        name,
                        type_style.paint("CAA"),
                        name_style.paint(url.as_str()),
                        caa.issuer_critical()
                    );
                }
                (tag, value) => {
                    println!(
                        "{} {} {:?} {:?} (critical={})",
                        name,
                        type_style.paint("CAA"),
                        tag,
                        value,
                        caa.issuer_critical()
                    );
                }
            }
        }
    }
}

async fn resolve_ptr(
    resolver: trust_dns_resolver::TokioAsyncResolver,
    addr: std::net::IpAddr,
    type_style: ansi_term::Style,
    name_style: ansi_term::Style,
) {
    if let Ok(resp) = resolver.reverse_lookup(addr).await {
        for ptr in resp {
            {
                println!(
                    "{} {} {}",
                    addr,
                    type_style.paint("PTR"),
                    name_style.paint(ptr.to_string())
                );
            }
        }
    } else {
        println!(
            "{} {} {}",
            addr,
            type_style.paint("PTR"),
            ansi_term::Color::Red.paint("NONE")
        );
    }
}
