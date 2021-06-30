use graphql_client::GraphQLQuery as _;
use structopt::StructOpt as _;

#[derive(structopt::StructOpt)]
struct Opt {
    #[structopt(short, long)]
    user: String,
    #[structopt(short, long)]
    from: Option<chrono::DateTime<chrono::Utc>>,
    #[structopt(short, long)]
    to: Option<chrono::DateTime<chrono::Utc>>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), anyhow::Error> {
    let opt = Opt::from_args();

    let token = std::env::var("GITHUB_ACCESS_TOKEN")?;
    let variables = query_contrib::Variables {
        login: opt.user,
        from: opt.from,
        to: opt.to,
    };
    let body = QueryContrib::build_query(variables);

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
    let resp: graphql_client::Response<query_contrib::ResponseData> = client
        .post("https://api.github.com/graphql")
        .json(&body)
        .send()
        .await?
        .json()
        .await?;
    if let Some(data) = resp.data {
        let contributions = data.user.unwrap().contributions_collection;
        println!(
            "Contributions during {} - {}",
            contributions.started_at, contributions.ended_at
        );

        println!("===== pull requests =====");
        for edge in contributions.pull_request_contributions.edges.unwrap() {
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
        for edge in contributions.issue_contributions.edges.unwrap() {
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
    } else {
        eprintln!("{:?}", resp.errors);
        std::process::exit(1);
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
