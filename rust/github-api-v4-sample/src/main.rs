#[derive(Debug, serde::Serialize)]
struct Claims {
    iat: u64,
    exp: u64,
    iss: u64,
}

#[derive(Debug, serde::Deserialize)]
struct AppInstallation {
    id: u64,
}

#[derive(Debug, serde::Deserialize)]
struct AppInstallationAccessToken {
    token: String,
}

#[derive(structopt::StructOpt)]
struct Opt {
    #[structopt(short, long)]
    private_key: String,
    #[structopt(short, long)]
    app_id: u64,
    #[structopt(short, long)]
    owner: String,
    #[structopt(short, long)]
    name: String,
}

type URI = String;
#[derive(graphql_client::GraphQLQuery)]
#[graphql(
    query_path = "latest_tarball_query.graphql",
    schema_path = "schema.docs.graphql",
    response_derives = "Debug"
)]
struct LatestTarballQuery;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::init();
    use structopt::StructOpt as _;
    let opt = Opt::from_args();

    let client = reqwest::Client::builder()
        .user_agent("github-api-v4-sample")
        .build()?;
    let pem = std::fs::read(&opt.private_key)?;
    let encoding_key = jsonwebtoken::EncodingKey::from_rsa_pem(&pem)?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();
    let claims = Claims {
        iat: now,
        exp: now + 9 * 60,
        iss: opt.app_id,
    };
    let jwt = jsonwebtoken::encode(
        &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256),
        &claims,
        &encoding_key,
    )?;

    let installations: Vec<AppInstallation> = client
        .get("https://api.github.com/app/installations")
        .bearer_auth(&jwt)
        .header(reqwest::header::ACCEPT, "application/vnd.github.v3+json")
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let installation_id = installations[0].id;

    let AppInstallationAccessToken { token } = client
        .post(&format!(
            "https://api.github.com/app/installations/{}/access_tokens",
            installation_id
        ))
        .bearer_auth(&jwt)
        .header(reqwest::header::ACCEPT, "application/vnd.github.v3+json")
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    use graphql_client::GraphQLQuery as _;
    let query = LatestTarballQuery::build_query(latest_tarball_query::Variables {
        owner: opt.owner,
        name: opt.name,
    });
    let resp: graphql_client::Response<latest_tarball_query::ResponseData> = client
        .post("https://api.github.com/graphql")
        .bearer_auth(&token)
        .json(&query)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    if let Some(errors) = resp.errors {
        return Err(anyhow::anyhow!("GraphQL error: {:?}", errors));
    }
    if let Some(
        latest_tarball_query::LatestTarballQueryRepositoryDefaultBranchRefTargetOn::Commit(commit),
    ) = resp
        .data
        .as_ref()
        .and_then(|data| data.repository.as_ref())
        .and_then(|repo| repo.default_branch_ref.as_ref())
        .and_then(|r| r.target.as_ref())
        .map(|target| &target.on)
    {
        println!("{}", commit.tarball_url);
    } else {
        return Err(anyhow::anyhow!("Unexpected query response: {:?}", resp));
    }
    Ok(())
}
