use clap::Parser as _;
use graphql_client::GraphQLQuery as _;

#[derive(clap::Parser)]
struct Opt {
    #[clap(short, long)]
    user: String,
    #[clap(short, long)]
    from: Option<chrono::NaiveDate>,
    #[clap(short, long)]
    to: Option<chrono::NaiveDate>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), anyhow::Error> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "info");
    }
    tracing_subscriber::fmt::init();
    let opt = Opt::parse();

    let token = std::env::var("GITHUB_ACCESS_TOKEN")?;
    let mut from = opt.from.map(|from| {
        chrono::DateTime::from_utc(
            from.and_time(chrono::NaiveTime::from_hms(0, 0, 0)),
            chrono::Utc,
        )
    });
    let mut to = opt.to.map(|to| {
        chrono::DateTime::from_utc(
            to.and_time(chrono::NaiveTime::from_hms(0, 0, 0)),
            chrono::Utc,
        )
    });
    let variables = query_contrib::Variables {
        login: opt.user.clone(),
        from,
        to,
    };
    let mut body = QueryContrib::build_query(variables);

    let client = reqwest::Client::builder()
        .user_agent("https://github.com/eagletmt/misc/rust/gh-contrib")
        .default_headers(
            std::iter::once((
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token))?,
            ))
            .collect(),
        )
        .build()?;

    let min_to = chrono::Utc::now() - chrono::Duration::days(365 * 11);
    while to.map(|t| t > min_to).unwrap_or(true) {
        let resp: graphql_client::Response<query_contrib::ResponseData> = client
            .post("https://api.github.com/graphql")
            .json(&body)
            .send()
            .await?
            .json()
            .await?;
        if let Some(data) = resp.data {
            let contributions = data.user.unwrap().contributions_collection;
            let pull_request_contributions =
                contributions.pull_request_contributions.edges.unwrap();
            let issue_contributions = contributions.issue_contributions.edges.unwrap();
            if pull_request_contributions.is_empty() && issue_contributions.is_empty() {
                tracing::warn!(
                    started_at = tracing::field::display(&contributions.started_at),
                    ended_at = tracing::field::display(&contributions.ended_at),
                    "No contributions found",
                );
                to = Some(contributions.started_at);
                from = if let Some(f) = from {
                    if f < contributions.started_at {
                        Some(f)
                    } else {
                        None
                    }
                } else {
                    None
                };
                let variables = query_contrib::Variables {
                    login: opt.user.clone(),
                    from,
                    to,
                };
                body = QueryContrib::build_query(variables);
                continue;
            }

            println!(
                "Contributions during {} - {}",
                contributions.started_at, contributions.ended_at
            );

            println!("===== pull requests =====");
            for edge in pull_request_contributions {
                let pull_request = edge.unwrap().node.unwrap().pull_request;
                let state = match pull_request.state {
                    query_contrib::PullRequestState::CLOSED => {
                        ansi_term::Color::Red.paint("Closed").to_string()
                    }
                    query_contrib::PullRequestState::MERGED => {
                        ansi_term::Color::Purple.paint("Merged").to_string()
                    }
                    query_contrib::PullRequestState::OPEN => {
                        ansi_term::Color::Green.paint("Open").to_string()
                    }
                    query_contrib::PullRequestState::Other(s) => s,
                };
                println!(
                    "[{}] {} [{}]",
                    state, pull_request.title, pull_request.created_at
                );
                println!("  {}", pull_request.url);
            }

            println!("===== issues =====");
            for edge in issue_contributions {
                let issue = edge.unwrap().node.unwrap().issue;
                let state = match issue.state {
                    query_contrib::IssueState::CLOSED => {
                        ansi_term::Color::Red.paint("Closed").to_string()
                    }
                    query_contrib::IssueState::OPEN => {
                        ansi_term::Color::Green.paint("Open").to_string()
                    }
                    query_contrib::IssueState::Other(s) => s,
                };
                println!("[{}] {} [{}]", state, issue.title, issue.created_at);
                println!("  {}", issue.url);
            }
            break;
        } else {
            eprintln!("{:?}", resp.errors);
            std::process::exit(1);
        }
    }
    Ok(())
}

#[derive(graphql_client::GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/query_contrib.graphql",
    response_derives = "Debug"
)]
struct QueryContrib;
type DateTime = chrono::DateTime<chrono::Utc>;
#[allow(clippy::upper_case_acronyms)]
type URI = String;
