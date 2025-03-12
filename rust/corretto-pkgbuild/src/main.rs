#[derive(Debug, clap::Parser, serde::Serialize)]
struct Args {
    /// JDK version
    #[clap(short, long)]
    jdk_version: u8,
    /// pkgrel
    #[clap(short, long, default_value_t = 1)]
    pkgrel: u8,
    /// Maintainer header name
    #[clap(short, long)]
    maintainer: Option<String>,
}

#[derive(Debug, PartialEq, serde::Serialize)]
struct Tarballs {
    x86_64_download: String,
    x86_64_checksum_sha256: String,
    aarch64_download: String,
    aarch64_checksum_sha256: String,
}

#[derive(Debug, serde::Serialize)]
struct Pkgbuild {
    #[serde(flatten)]
    args: Args,
    pkgver: String,
    #[serde(flatten)]
    tarballs: Tarballs,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    use clap::Parser as _;
    let args = Args::parse();

    let mut handlebars = handlebars::Handlebars::new();
    handlebars.set_strict_mode(true);
    handlebars.register_escape_fn(handlebars::no_escape);
    handlebars.register_template_string("PKGBUILD", include_str!("template/PKGBUILD"))?;

    let octocrab = octocrab::instance();
    let mut page = octocrab
        .repos("corretto", format!("corretto-{}", args.jdk_version))
        .releases()
        .list()
        .per_page(1)
        .page(1u32)
        .send()
        .await?;

    let pkgbuild = loop {
        let release = page
            .items
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("No more releases"))?;
        if release.draft {
            tracing::info!("Skip draft release: {}", release.html_url);
        } else if release.prerelease {
            tracing::info!("Skip pre-release: {}", release.html_url);
        } else if let Some(body) = release.body {
            if let Some(tarballs) = extract_tarballs(&body)? {
                break Pkgbuild {
                    args,
                    pkgver: release.tag_name,
                    tarballs,
                };
            }
        }

        page = octocrab
            .get_page(&page.next)
            .await?
            .ok_or_else(|| anyhow::anyhow!("No more releases"))?;
    };

    let stdout = std::io::stdout().lock();
    handlebars.render_to_write("PKGBUILD", &pkgbuild, stdout)?;

    Ok(())
}

#[derive(Debug)]
enum Node<'a> {
    Text(pulldown_cmark::CowStr<'a>),
    Code(pulldown_cmark::CowStr<'a>),
    Html(pulldown_cmark::CowStr<'a>),
    Element(Element<'a>),
}

#[derive(Debug)]
struct Element<'a> {
    tag: pulldown_cmark::Tag<'a>,
    children: Vec<Node<'a>>,
}

fn skip_until_tag<'a, F>(parser: &mut pulldown_cmark::Parser<'a>, f: F) -> anyhow::Result<bool>
where
    F: Fn(pulldown_cmark::Tag<'a>) -> bool,
{
    let mut depth = 0;
    for event in parser {
        match event {
            pulldown_cmark::Event::Start(tag) => {
                if depth == 0 && f(tag) {
                    return Ok(true);
                } else {
                    depth += 1;
                }
            }
            pulldown_cmark::Event::End(_) => {
                depth -= 1;
            }
            _ => {}
        }
    }
    Ok(false)
}

fn extract_tarballs(body: &str) -> anyhow::Result<Option<Tarballs>> {
    let mut parser = pulldown_cmark::Parser::new_ext(body, pulldown_cmark::Options::ENABLE_TABLES);
    anyhow::ensure!(
        skip_until_tag(&mut parser, |tag| {
            matches!(tag, pulldown_cmark::Tag::Table(_))
        })?,
        "Table element is not found"
    );
    let table = build_nodes(&mut parser);
    let rows = table.into_iter().filter_map(|n| {
        if let Node::Element(Element {
            tag: pulldown_cmark::Tag::TableRow,
            children: cells,
        }) = n
        {
            Some(cells)
        } else {
            None
        }
    });

    #[derive(Debug)]
    struct Entry {
        download: String,
        checksum_sha256: String,
    }

    let mut x86_64_entry = None;
    let mut aarch64_entry = None;
    for row in rows {
        let mut cells = row.into_iter().filter_map(|n| {
            if let Node::Element(Element {
                tag: pulldown_cmark::Tag::TableCell,
                children: texts,
            }) = n
            {
                Some(texts)
            } else {
                None
            }
        });
        let platform = textify(
            cells
                .next()
                .ok_or_else(|| anyhow::anyhow!("Platform cell is not found"))?,
        );
        let _type = cells
            .next()
            .ok_or_else(|| anyhow::anyhow!("Type cell is not found"))?;
        let download = cells
            .next()
            .ok_or_else(|| anyhow::anyhow!("Download cell is not found"))?;
        let checksum = cells
            .next()
            .ok_or_else(|| anyhow::anyhow!("Checksum cell is not found"))?;
        let sigfile = cells
            .next()
            .ok_or_else(|| anyhow::anyhow!("Sig File cell is not found"))?;
        if !sigfile.is_empty() {
            let download = download
                .into_iter()
                .find_map(|n| {
                    if let Node::Element(Element {
                        tag: pulldown_cmark::Tag::Link { dest_url, .. },
                        ..
                    }) = n
                    {
                        Some(dest_url)
                    } else {
                        None
                    }
                })
                .ok_or_else(|| anyhow::anyhow!("Link element is not found in Download cell"))?;
            let checksum_sha256 = checksum
                .into_iter()
                .rev()
                .find_map(|n| if let Node::Code(s) = n { Some(s) } else { None })
                .ok_or_else(|| anyhow::anyhow!("Code node is not found in Checksum cell"))?;
            if &*platform == "Linux x64" {
                x86_64_entry = Some(Entry {
                    download: download.into_string(),
                    checksum_sha256: checksum_sha256.into_string(),
                });
            } else if &*platform == "Linux aarch64" {
                aarch64_entry = Some(Entry {
                    download: download.into_string(),
                    checksum_sha256: checksum_sha256.into_string(),
                });
            }
        }
    }

    if let (Some(x86_64_entry), Some(aarch64_entry)) = (x86_64_entry, aarch64_entry) {
        Ok(Some(Tarballs {
            x86_64_download: x86_64_entry.download,
            x86_64_checksum_sha256: x86_64_entry.checksum_sha256,
            aarch64_download: aarch64_entry.download,
            aarch64_checksum_sha256: aarch64_entry.checksum_sha256,
        }))
    } else {
        Ok(None)
    }
}

fn textify(nodes: Vec<Node>) -> tendril::StrTendril {
    let mut ret = tendril::StrTendril::new();
    for node in nodes {
        match node {
            Node::Element(Element { children, .. }) => {
                ret.push_tendril(&textify(children));
            }
            Node::Text(s) | Node::Code(s) | Node::Html(s) => {
                ret.push_slice(&s);
            }
        }
    }
    ret
}

fn build_nodes<'a>(parser: &mut pulldown_cmark::Parser<'a>) -> Vec<Node<'a>> {
    let mut nodes = Vec::new();
    while let Some(event) = parser.next() {
        match event {
            pulldown_cmark::Event::Start(tag) => {
                let children = build_nodes(parser);
                nodes.push(Node::Element(Element { tag, children }));
            }
            pulldown_cmark::Event::End(_) => {
                return nodes;
            }
            pulldown_cmark::Event::Text(s) => {
                nodes.push(Node::Text(s));
            }
            pulldown_cmark::Event::Code(s) => {
                nodes.push(Node::Code(s));
            }
            pulldown_cmark::Event::InlineHtml(s) => {
                nodes.push(Node::Html(s));
            }
            e => {
                todo!("{:?}", e)
            }
        }
    }
    nodes
}

#[cfg(test)]
mod test {
    #[test]
    fn extract_tarballs_from_release() {
        let body = include_str!("../test/corretto-8.342.07.4.md");
        let tarballs = super::extract_tarballs(body).unwrap().unwrap();
        assert_eq!(tarballs.x86_64_download, "https://corretto.aws/downloads/resources/8.342.07.4/amazon-corretto-8.342.07.4-linux-x64.tar.gz");
        assert_eq!(
            tarballs.x86_64_checksum_sha256,
            "f10fc46f42df58cf26a4689a7016aa610b691ad4e8be7c349f8651dec79d4e41"
        );
        assert_eq!(tarballs.aarch64_download, "https://corretto.aws/downloads/resources/8.342.07.4/amazon-corretto-8.342.07.4-linux-aarch64.tar.gz");
        assert_eq!(
            tarballs.aarch64_checksum_sha256,
            "2d454c4804fc2ee5a2aef9f517ca6c2b85dee7728d74edf20f85a35681b2d143"
        );
    }

    #[test]
    fn return_none_if_no_linux_platforms() {
        let body = include_str!("../test/corretto-11.0.16.8.3.md");
        let tarballs = super::extract_tarballs(body).unwrap();
        assert_eq!(tarballs, None);
    }

    #[test]
    fn table_without_leading_empty_line() {
        let body = include_str!("../test/corretto-11.0.20.9.1.md");
        let tarballs = super::extract_tarballs(body).unwrap().unwrap();
        assert_eq!(tarballs.x86_64_download, "https://corretto.aws/downloads/resources/11.0.20.9.1/amazon-corretto-11.0.20.9.1-linux-x64.tar.gz");
        assert_eq!(
            tarballs.x86_64_checksum_sha256,
            "b6150255d304eab8fdcc0422beab277e5395bc481b4f87f096da78a979e47d47"
        );
        assert_eq!(tarballs.aarch64_download, "https://corretto.aws/downloads/resources/11.0.20.9.1/amazon-corretto-11.0.20.9.1-linux-aarch64.tar.gz");
        assert_eq!(
            tarballs.aarch64_checksum_sha256,
            "17c33bd5fb51fd8b4b5cdfce9d656f31698a6c6ccf018f4f2bf99d714948c736"
        );
    }
}
