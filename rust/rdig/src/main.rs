extern crate ansi_term;
extern crate resolv;

fn main() {
    let mut resolver = resolv::Resolver::new().expect("Failed to initialize libresolv");

    for arg in std::env::args().skip(1) {
        resolve_name(&mut resolver, arg);
    }
}

fn resolve_name(resolver: &mut resolv::Resolver, name: String) {
    let type_style = ansi_term::Style::new().fg(ansi_term::Color::Yellow);
    let name_style = ansi_term::Style::new().fg(ansi_term::Color::Green);
    let addr_style = ansi_term::Style::new().fg(ansi_term::Color::Blue);

    if let Ok(mut resp) = resolver.query(name.as_bytes(), resolv::Class::IN, resolv::RecordType::MX)
    {
        for mx in resp.answers::<resolv::record::MX>() {
            println!(
                "{} {} {} {}",
                name,
                type_style.paint("MX"),
                name_style.paint(mx.data.exchange),
                mx.data.preference
            );
        }
    }

    if let Ok(mut resp) =
        resolver.query(name.as_bytes(), resolv::Class::IN, resolv::RecordType::TXT)
    {
        for txt in resp.answers::<resolv::record::TXT>() {
            println!("{} {} {}", name, type_style.paint("TXT"), txt.data.dname);
        }
    }

    let mut name = name;
    loop {
        let mut resolved = false;
        if let Ok(mut resp) = resolver.query(
            name.as_bytes(),
            resolv::Class::IN,
            resolv::RecordType::CNAME,
        ) {
            for cname in resp.answers::<resolv::record::CNAME>() {
                resolved = true;
                println!(
                    "{} {} {}",
                    name,
                    type_style.paint("CNAME"),
                    name_style.paint(&*cname.data.cname)
                );
                name = cname.data.cname;
            }
        }
        if !resolved {
            break;
        }
    }
    let name = name;

    let mut addrs = Vec::new();
    if let Ok(mut resp) = resolver.query(name.as_bytes(), resolv::Class::IN, resolv::RecordType::A)
    {
        for a in resp.answers::<resolv::record::A>() {
            let addr = a.data.address.to_string();
            println!(
                "{} {} {}",
                name,
                type_style.paint("A"),
                addr_style.paint(&*addr)
            );
            let octets = a.data.address.octets();
            addrs.push((
                addr,
                format!(
                    "{}.{}.{}.{}.in-addr.arpa",
                    octets[3], octets[2], octets[1], octets[0]
                ),
            ));
        }
    }

    if let Ok(mut resp) =
        resolver.query(name.as_bytes(), resolv::Class::IN, resolv::RecordType::AAAA)
    {
        for aaaa in resp.answers::<resolv::record::AAAA>() {
            let addr = aaaa.data.address.to_string();
            println!(
                "{} {} {}",
                name,
                type_style.paint("AAAA"),
                addr_style.paint(&*addr)
            );
            let segments = aaaa.data.address.segments();
            let mut buf = String::new();
            for i in 0..8 {
                let mut seg = segments[7 - i];
                for _ in 0..4 {
                    use std::fmt::Write;
                    write!(&mut buf, "{:x}.", seg % 16).unwrap();
                    seg /= 16;
                }
            }
            buf.push_str("ip6.arpa");
            addrs.push((addr, buf));
        }
    }

    for (addr, rev_addr) in addrs {
        if let Ok(mut resp) = resolver.query(
            rev_addr.as_bytes(),
            resolv::Class::IN,
            resolv::RecordType::PTR,
        ) {
            for ptr in resp.answers::<resolv::record::PTR>() {
                {
                    println!(
                        "{} {} {}",
                        addr,
                        type_style.paint("PTR"),
                        name_style.paint(ptr.data.dname)
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
