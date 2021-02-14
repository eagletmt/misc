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
    let folderid = std::env::var("ENVWARDEN_FOLDERID").unwrap_or_else(|_| {
        eprintln!("ENVWARDEN_FOLDERID must be set");
        std::process::exit(1);
    });

    let output = std::process::Command::new("bw")
        .arg("list")
        .arg("items")
        .arg(&name)
        .arg("--folderid")
        .arg(folderid)
        .output()?;
    if !output.status.success() {
        eprintln!("`bw list items` failed");
        std::io::stdout().write_all(&output.stdout)?;
        std::io::stderr().write_all(&output.stderr)?;
        std::process::exit(output.status.code().unwrap_or(1));
    }

    let items: Vec<Item> = serde_json::from_slice(&output.stdout)?;
    let mut envs = Vec::new();
    for item in items.into_iter() {
        if item.name == name {
            match item.type_ {
                ItemType::SecureNote => {
                    for field in item.fields {
                        match field.type_ {
                            FieldType::Text | FieldType::Hidden => {
                                envs.push((field.name, field.value));
                            }
                            _ => {
                                eprintln!(
                                    "{}: ignoring field {} in item {}",
                                    me, field.name, item.name
                                );
                            }
                        }
                    }
                }
                _ => {
                    eprintln!("{}: ignoring item {}", me, item.name);
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
struct Item {
    #[serde(rename = "type")]
    type_: ItemType,
    name: String,
    fields: Vec<ItemField>,
}

// https://bitwarden.com/help/article/cli/#enums
#[derive(Debug, serde_repr::Deserialize_repr)]
#[repr(u8)]
enum ItemType {
    Login = 1,
    SecureNote = 2,
    Card = 3,
    Identity = 4,
}

#[derive(Debug, serde::Deserialize)]
struct ItemField {
    #[serde(rename = "type")]
    type_: FieldType,
    name: String,
    value: String,
}

// https://bitwarden.com/help/article/cli/#enums
#[derive(Debug, serde_repr::Deserialize_repr)]
#[repr(u8)]
enum FieldType {
    Text = 0,
    Hidden = 1,
    Boolean = 2,
}
