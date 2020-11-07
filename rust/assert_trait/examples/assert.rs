use futures::StreamExt as _;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    for c in assert_trait::assert_trait!(Iterator<Item = char>, "assert".chars()) {
        println!("{}", c);
    }
    for line in assert_trait::assert_trait!(<'a>, Iterator<Item = &'a str>, "a\nb\nc".lines()) {
        println!("{}", line);
    }

    let interval = assert_trait::assert_trait!(
        futures::Stream + Unpin,
        tokio::time::interval(tokio::time::Duration::from_secs(2))
            .map(|i| futures::future::Either::Left(i))
            .take(4)
    );
    let status = futures::stream::once(tokio::process::Command::new("sleep").arg("6").status())
        .map(|s| futures::future::Either::Right(s));
    // OK
    let status = assert_trait::assert_trait!(futures::Stream, status);
    // NG
    // let status = assert_trait::assert_trait!(Unpin, status);
    tokio::pin!(status);
    // OK
    let status = assert_trait::assert_trait!(Unpin, status);
    let mut stream = assert_trait::assert_trait!(
        futures::Stream<Item = futures::future::Either<tokio::time::Instant, tokio::io::Result<std::process::ExitStatus>>> + Unpin,
        futures::stream::select(interval, status)
    );
    while let Some(item) = stream.next().await {
        println!("{:?}", item);
    }

    Ok(())
}
