use std::io::Write as _;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args();
    let me = args.next().unwrap();
    let name = args.next().unwrap_or_else(|| {
        eprintln!("Usage: {} NAME PROG ARGS...", me);
        std::process::exit(1);
    });
    let prog = args.next().unwrap_or_else(|| {
        eprintln!("Usage: {} NAME PROG ARGS...", me);
        std::process::exit(1);
    });
    let tags = std::env::var("ENVOP_TAGS").unwrap_or_else(|_| "envop".to_owned());
    let vault = std::env::var("ENVOP_VAULT").unwrap_or_else(|_| "Private".to_owned());

    let output = std::process::Command::new("op")
        .arg("list")
        .arg("items")
        .arg("--vault")
        .arg(&vault)
        .arg("--categories")
        .arg("Secure Note")
        .arg("--tags")
        .arg(&tags)
        .output()?;
    if !output.status.success() {
        eprintln!("`op list items` failed");
        std::io::stdout().write_all(&output.stdout)?;
        std::io::stderr().write_all(&output.stderr)?;
        std::process::exit(output.status.code().unwrap_or(1));
    }

    let item_summaries: Vec<ItemSummary> = serde_json::from_slice(&output.stdout)?;
    let mut envs = Vec::new();
    for item_summary in item_summaries
        .into_iter()
        .filter(|item_summary| item_summary.overview.title == name)
    {
        let output = std::process::Command::new("op")
            .arg("get")
            .arg("item")
            .arg("--vault")
            .arg(&vault)
            .arg(&item_summary.uuid)
            .output()?;
        if !output.status.success() {
            eprintln!("`op get item {}` failed", item_summary.uuid);
            std::io::stdout().write_all(&output.stdout)?;
            std::io::stderr().write_all(&output.stderr)?;
            std::process::exit(output.status.code().unwrap_or(1));
        }
        let item: Item = serde_json::from_slice(&output.stdout)?;
        for section in item.details.sections.into_iter() {
            for field in section.fields.into_iter() {
                if field.k == "string" || field.k == "concealed" {
                    envs.push((field.t, field.v));
                } else {
                    eprintln!(
                        "{}: ignoring field {} in item {}",
                        me, field.t, item_summary.uuid
                    );
                }
            }
        }
    }

    let mut cmd = std::process::Command::new(&prog);
    cmd.envs(envs).args(args);
    let status = exec(cmd)?;
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
    Ok(())
}

#[cfg(unix)]
fn exec(
    mut cmd: std::process::Command,
) -> Result<std::process::ExitStatus, Box<dyn std::error::Error>> {
    use std::os::unix::process::CommandExt as _;
    Err(Box::new(cmd.exec()))
}
#[cfg(windows)]
fn exec(
    mut cmd: std::process::Command,
) -> Result<std::process::ExitStatus, Box<dyn std::error::Error>> {
    Ok(cmd.status()?)
}

#[derive(Debug, serde::Deserialize)]
struct ItemSummary {
    uuid: String,
    overview: ItemOverview,
}

#[derive(Debug, serde::Deserialize)]
struct ItemOverview {
    title: String,
}

#[derive(Debug, serde::Deserialize)]
struct Item {
    details: ItemDetails,
}

#[derive(Debug, serde::Deserialize)]
struct ItemDetails {
    sections: Vec<ItemSection>,
}

#[derive(Debug, serde::Deserialize)]
struct ItemSection {
    fields: Vec<ItemField>,
}

#[derive(Debug, serde::Deserialize)]
struct ItemField {
    k: String,
    t: String,
    v: String,
}
