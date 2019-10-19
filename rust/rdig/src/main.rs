fn main() -> Result<(), failure::Error> {
    let mut resolver = trust_dns_resolver::Resolver::default()?;

    for arg in std::env::args().skip(1) {
        resolve_name(&mut resolver, arg);
    }
    Ok(())
}

fn resolve_name(resolver: &mut trust_dns_resolver::Resolver, name: String) {
    let type_style = ansi_term::Style::new().fg(ansi_term::Color::Yellow);
    let name_style = ansi_term::Style::new().fg(ansi_term::Color::Green);
    let addr_style = ansi_term::Style::new().fg(ansi_term::Color::Blue);

    if let Ok(resp) = resolver.mx_lookup(&name) {
        for mx in resp {
            println!(
                "{} {} {} {}",
                name,
                type_style.paint("MX"),
                name_style.paint(mx.exchange().to_utf8()),
                mx.preference()
            );
        }
    }

    if let Ok(resp) = resolver.txt_lookup(&name) {
        for txt in resp {
            for data in txt.txt_data() {
                println!(
                    "{} {} {:?}",
                    name,
                    type_style.paint("TXT"),
                    String::from_utf8_lossy(&data),
                );
            }
        }
    }

    let mut name = name;
    loop {
        let mut resolved = false;
        if let Ok(resp) = resolver.lookup(&name, trust_dns_proto::rr::RecordType::CNAME) {
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
    if let Ok(resp) = resolver.ipv4_lookup(&name) {
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

    if let Ok(resp) = resolver.ipv6_lookup(&name) {
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

    for addr in addrs {
        if let Ok(resp) = resolver.reverse_lookup(addr) {
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
}
