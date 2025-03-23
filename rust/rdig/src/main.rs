#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let mut resolver = hickory_resolver::Resolver::builder_with_config(
        hickory_resolver::config::ResolverConfig::cloudflare_https(),
        hickory_resolver::name_server::TokioConnectionProvider::default(),
    )
    .build();

    for arg in std::env::args().skip(1) {
        resolve_name(&mut resolver, arg).await?;
    }
    Ok(())
}

async fn resolve_name<P>(
    resolver: &mut hickory_resolver::Resolver<P>,
    name: String,
) -> Result<(), Box<dyn std::error::Error>>
where
    P: hickory_resolver::name_server::ConnectionProvider,
{
    const TYPE_STYLE: anstyle::Style =
        anstyle::Style::new().fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Yellow)));
    const NAME_STYLE: anstyle::Style =
        anstyle::Style::new().fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Green)));
    const ADDR_STYLE: anstyle::Style =
        anstyle::Style::new().fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Blue)));

    let mx_handle = tokio::spawn(resolve_mx(
        resolver.clone(),
        name.clone(),
        TYPE_STYLE,
        NAME_STYLE,
    ));
    let txt_handle = tokio::spawn(resolve_txt(resolver.clone(), name.clone(), TYPE_STYLE));
    let caa_handle = tokio::spawn(resolve_caa(
        resolver.clone(),
        name.clone(),
        TYPE_STYLE,
        NAME_STYLE,
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
                hickory_resolver::proto::rr::RecordType::CNAME,
            )
            .await
        {
            for cname in resp {
                resolved = true;
                let next_name = cname.as_cname().unwrap().to_string();
                anstream::println!(
                    "{} {}CNAME{} {}{}{}",
                    name,
                    TYPE_STYLE.render(),
                    TYPE_STYLE.render_reset(),
                    NAME_STYLE.render(),
                    next_name,
                    NAME_STYLE.render_reset(),
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
            anstream::println!(
                "{} {}A{} {}{}{}",
                name,
                TYPE_STYLE.render(),
                TYPE_STYLE.render_reset(),
                ADDR_STYLE.render(),
                a.to_string(),
                ADDR_STYLE.render_reset(),
            );
            addrs.push(std::net::IpAddr::from(*a));
        }
    }

    if let Ok(resp) = resolver.ipv6_lookup(name.as_str()).await {
        for aaaa in resp {
            anstream::println!(
                "{} {}AAAA{} {}{}{}",
                name,
                TYPE_STYLE.render(),
                TYPE_STYLE.render_reset(),
                ADDR_STYLE.render(),
                aaaa.to_string(),
                ADDR_STYLE.render_reset(),
            );
            addrs.push(std::net::IpAddr::from(*aaaa));
        }
    }

    let mut addr_handles = Vec::new();
    for addr in addrs {
        addr_handles.push(tokio::spawn(resolve_ptr(
            resolver.clone(),
            addr,
            TYPE_STYLE,
            NAME_STYLE,
        )));
    }
    for handle in addr_handles {
        handle.await?;
    }
    Ok(())
}

async fn resolve_mx<P>(
    resolver: hickory_resolver::Resolver<P>,
    name: String,
    type_style: anstyle::Style,
    name_style: anstyle::Style,
) where
    P: hickory_resolver::name_server::ConnectionProvider,
{
    if let Ok(resp) = resolver.mx_lookup(name.as_str()).await {
        let mut records: Vec<_> = resp.into_iter().collect();
        records.sort_unstable_by(|x, y| {
            x.preference()
                .cmp(&y.preference())
                .then_with(|| x.exchange().cmp(y.exchange()))
        });
        for mx in records {
            anstream::println!(
                "{} {}MX{} {}{}{} {}",
                name,
                type_style.render(),
                type_style.render_reset(),
                name_style.render(),
                mx.exchange().to_utf8(),
                name_style.render_reset(),
                mx.preference()
            );
        }
    }
}

async fn resolve_txt<P>(
    resolver: hickory_resolver::Resolver<P>,
    name: String,
    type_style: anstyle::Style,
) where
    P: hickory_resolver::name_server::ConnectionProvider,
{
    if let Ok(resp) = resolver.txt_lookup(name.as_str()).await {
        for txt in resp {
            for data in txt.txt_data() {
                anstream::println!(
                    "{} {}TXT{} {}",
                    name,
                    type_style.render(),
                    type_style.render_reset(),
                    String::from_utf8_lossy(data),
                );
            }
        }
    }
}

async fn resolve_caa<P>(
    resolver: hickory_resolver::Resolver<P>,
    name: String,
    type_style: anstyle::Style,
    name_style: anstyle::Style,
) where
    P: hickory_resolver::name_server::ConnectionProvider,
{
    if let Ok(resp) = resolver
        .lookup(name.as_str(), hickory_resolver::proto::rr::RecordType::CAA)
        .await
    {
        for rdata in resp {
            let caa = rdata.as_caa().unwrap();
            use hickory_resolver::proto::rr::rdata::caa::{Property, Value};
            match (caa.tag(), caa.value()) {
                (Property::Issue, Value::Issuer(Some(domain), _)) => {
                    anstream::println!(
                        "{} {}CAA{} issue {}{}{} (critical={})",
                        name,
                        type_style.render(),
                        type_style.render_reset(),
                        name_style.render(),
                        domain.to_utf8(),
                        name_style.render_reset(),
                        caa.issuer_critical()
                    );
                }
                (Property::IssueWild, Value::Issuer(Some(domain), _)) => {
                    anstream::println!(
                        "{} {}CAA{} issuewild {}{}{} (critical={})",
                        name,
                        type_style.render(),
                        type_style.render_reset(),
                        name_style.render(),
                        domain.to_utf8(),
                        name_style.render_reset(),
                        caa.issuer_critical()
                    );
                }
                (Property::Iodef, Value::Url(url)) => {
                    anstream::println!(
                        "{} {}CAA{} iodef {}{}{} (critical={})",
                        name,
                        type_style.render(),
                        type_style.render_reset(),
                        name_style.render(),
                        url.as_str(),
                        name_style.render_reset(),
                        caa.issuer_critical()
                    );
                }
                (tag, value) => {
                    anstream::println!(
                        "{} {}CAA{} {:?} {:?} (critical={})",
                        name,
                        type_style.render(),
                        type_style.render_reset(),
                        tag,
                        value,
                        caa.issuer_critical()
                    );
                }
            }
        }
    }
}

async fn resolve_ptr<P>(
    resolver: hickory_resolver::Resolver<P>,
    addr: std::net::IpAddr,
    type_style: anstyle::Style,
    name_style: anstyle::Style,
) where
    P: hickory_resolver::name_server::ConnectionProvider,
{
    if let Ok(resp) = resolver.reverse_lookup(addr).await {
        for ptr in resp {
            {
                anstream::println!(
                    "{} {}PTR{} {}{}{}",
                    addr,
                    type_style.render(),
                    type_style.render_reset(),
                    name_style.render(),
                    ptr.to_string(),
                    name_style.render_reset(),
                );
            }
        }
    } else {
        const NONE_STYLE: anstyle::Style =
            anstyle::Style::new().fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Red)));
        anstream::println!(
            "{} {}PTR{} {}NONE{}",
            addr,
            type_style.render(),
            type_style.render_reset(),
            NONE_STYLE.render(),
            NONE_STYLE.render_reset(),
        );
    }
}
