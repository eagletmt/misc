#[derive(Debug, clap::Parser)]
struct Args {
    // curl-compat options
    #[clap(short = 'H')]
    headers: Vec<String>,

    #[clap(long, default_value_t = false)]
    print: bool,
    #[clap(short, long)]
    output: String,

    url: String,
}

fn main() {
    use clap::Parser as _;
    let args = Args::parse();

    let mut cmd = vec![
        "ffmpeg".to_owned(),
        "-i".to_owned(),
        format!("'{}'", args.url),
        "-c:a".to_owned(),
        "copy".to_owned(),
        "-c:v".to_owned(),
        "copy".to_owned(),
        format!("'{}'", args.output),
    ];
    for header in args.headers {
        cmd.push("-headers".to_owned());
        cmd.push(format!("'{header}'"));
    }

    if args.print {
        println!("{}", cmd.join(" "));
    } else {
        use std::os::unix::process::CommandExt as _;
        let mut it = cmd.into_iter();
        let p = it.next().unwrap();
        let err = std::process::Command::new(p).args(it).exec();
        panic!("Failed to exec ffmpeg: {err}");
    }
}
