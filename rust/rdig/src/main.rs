extern crate resolv;

fn main() {
    let mut resolver = resolv::Resolver::new().expect("Failed to initialize libresolv");

    for arg in std::env::args().skip(1) {
        resolve_name(&mut resolver, arg);
    }
}

fn resolve_name(resolver: &mut resolv::Resolver, name: String) {
    if let Ok(mut resp) = resolver.query(name.as_bytes(), resolv::Class::IN, resolv::RecordType::MX)
    {
        for mx in resp.answers::<resolv::record::MX>() {
            println!("{} MX {} {}", name, mx.data.exchange, mx.data.preference);
        }
    }

    if let Ok(mut resp) =
        resolver.query(name.as_bytes(), resolv::Class::IN, resolv::RecordType::TXT)
    {
        for txt in resp.answers::<resolv::record::TXT>() {
            println!("{} TXT {}", name, txt.data.dname);
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
                println!("{} CNAME {}", name, cname.data.cname);
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
            println!("{} A {}", name, addr);
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
            println!("{} AAAA {}", name, addr);
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
                    println!("{} PTR {}", addr, ptr.data.dname);
                }
            }
        }
    }
}
