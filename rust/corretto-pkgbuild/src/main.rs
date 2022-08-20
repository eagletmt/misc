#[derive(Debug, clap::Parser)]
struct Args {
    /// JDK version
    #[clap(short, long)]
    jdk_version: u8,
    /// pkgrel
    #[clap(short, long, default_value_t = 1)]
    pkgrel: u8,
}

#[derive(Debug, serde::Serialize)]
struct Release {
    jdk_version: u8,
    pkgrel: u8,
    pkgver: String,
    x86_64_download: String,
    x86_64_checksum_sha256: String,
    aarch64_download: String,
    aarch64_checksum_sha256: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "info");
    }
    tracing_subscriber::fmt::init();
    use clap::Parser as _;
    let args = Args::parse();

    let mut handlebars = handlebars::Handlebars::new();
    handlebars.set_strict_mode(true);
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

    let release = loop {
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
            if let Some(r) = extract_release(&args, release.tag_name, &body)? {
                break r;
            }
        }

        page = octocrab
            .get_page(&page.next)
            .await?
            .ok_or_else(|| anyhow::anyhow!("No more releases"))?;
    };

    let stdout = std::io::stdout().lock();
    handlebars.render_to_write("PKGBUILD", &release, stdout)?;

    Ok(())
}

fn extract_release(args: &Args, tag_name: String, body: &str) -> anyhow::Result<Option<Release>> {
    let parser = pulldown_cmark::Parser::new_ext(body, pulldown_cmark::Options::ENABLE_TABLES);
    let mut body_html = String::new();
    pulldown_cmark::html::push_html(&mut body_html, parser);
    let fragment = scraper::Html::parse_fragment(&body_html);
    let row_selector = scraper::Selector::parse("table tbody tr").unwrap();
    let col_selector = scraper::Selector::parse("td").unwrap();
    let a_selector = scraper::Selector::parse("a[href]").unwrap();

    #[derive(Debug)]
    struct Entry {
        download: String,
        checksum_sha256: String,
    }
    let mut x86_64_entry = None;
    let mut aarch64_entry = None;
    for tr in fragment.select(&row_selector) {
        let mut tds = tr.select(&col_selector);
        let platform_td = tds
            .next()
            .ok_or_else(|| anyhow::anyhow!("Platform column is not found"))?;
        let platform = tendril::StrTendril::from_iter(platform_td.text());

        let _type_td = tds
            .next()
            .ok_or_else(|| anyhow::anyhow!("Type column is not found"))?;

        let download_td = tds
            .next()
            .ok_or_else(|| anyhow::anyhow!("Download Link column is not found"))?;
        let download_a = download_td
            .select(&a_selector)
            .next()
            .ok_or_else(|| anyhow::anyhow!("a tag in Download Link is not found"))?;
        let download = download_a.value().attr("href").unwrap();

        let checksum_td = tds
            .next()
            .ok_or_else(|| anyhow::anyhow!("Checksum column is not found"))?;
        let checksum = tendril::StrTendril::from_iter(checksum_td.text());
        let checksum_sha256 = checksum.rsplit(" / ").next().unwrap();

        if &*platform == "Linux x64" {
            x86_64_entry = Some(Entry {
                download: download.to_owned(),
                checksum_sha256: checksum_sha256.to_owned(),
            });
        } else if &*platform == "Linux aarch64" {
            aarch64_entry = Some(Entry {
                download: download.to_owned(),
                checksum_sha256: checksum_sha256.to_owned(),
            });
        }
    }

    if let (Some(x86_64_entry), Some(aarch64_entry)) = (x86_64_entry, aarch64_entry) {
        Ok(Some(Release {
            jdk_version: args.jdk_version,
            pkgrel: args.pkgrel,
            pkgver: tag_name,
            x86_64_download: x86_64_entry.download,
            x86_64_checksum_sha256: x86_64_entry.checksum_sha256,
            aarch64_download: aarch64_entry.download,
            aarch64_checksum_sha256: aarch64_entry.checksum_sha256,
        }))
    } else {
        Ok(None)
    }
}
