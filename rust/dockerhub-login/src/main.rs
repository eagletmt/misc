use futures_util::SinkExt as _;
use futures_util::StreamExt as _;

#[derive(Debug, clap::Parser)]
struct Args {
    #[clap(short, long, env = "DOCKERHUB_USERNAME")]
    username: String,
    #[clap(short, long, env = "DOCKERHUB_PASSWORD")]
    password: String,
}

#[derive(Debug, serde::Deserialize)]
struct User {
    username: String,
    profile_url: String,
    gravatar_url: String,
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

    let auth_header = obtain_auth_header(&args.username, &args.password).await?;
    let client = reqwest::Client::new();
    let user: User = client
        .get("https://hub.docker.com/v2/user/")
        .header(reqwest::header::AUTHORIZATION, auth_header)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    println!("username: {}", user.username);
    println!("profile_url: {}", user.profile_url);
    println!("gravatar_url: {}", user.gravatar_url);

    Ok(())
}

async fn obtain_auth_header(username: &str, password: &str) -> anyhow::Result<String> {
    let (mut child, mut ws) = dockerhub_login::browser::launch().await?;

    let resp: dockerhub_login::cdp::Response<dockerhub_login::target::GetTargetsResult> =
        dockerhub_login::send(
            &mut ws,
            dockerhub_login::cdp::Request {
                id: 1,
                session_id: None,
                method: "Target.getTargets",
                params: (),
            },
        )
        .await?;
    let target_info = resp
        .result
        .target_infos
        .into_iter()
        .find(|t| t.type_ == "page")
        .unwrap();

    let resp: dockerhub_login::cdp::Response<dockerhub_login::target::AttachToTargetResult> =
        dockerhub_login::send(
            &mut ws,
            dockerhub_login::cdp::Request {
                id: 2,
                session_id: None,
                method: "Target.attachToTarget",
                params: dockerhub_login::target::AttachToTargetParams {
                    target_id: &target_info.target_id,
                    flatten: true,
                },
            },
        )
        .await?;
    let session_id = resp.result.session_id;

    let mut sequencer = 1..;
    let _: dockerhub_login::cdp::Response<serde_json::Value> = dockerhub_login::send(
        &mut ws,
        dockerhub_login::cdp::Request {
            id: sequencer.next().unwrap(),
            session_id: Some(&session_id),
            method: "Page.navigate",
            params: dockerhub_login::page::NavigateParams {
                url: "https://hub.docker.com",
            },
        },
    )
    .await?;

    let log_in_node_id =
        dockerhub_login::find_element(&mut ws, &session_id, &mut sequencer, "#log_in").await?;
    dockerhub_login::click_element(&mut ws, &session_id, &mut sequencer, log_in_node_id).await?;

    let username_node_id =
        dockerhub_login::find_element(&mut ws, &session_id, &mut sequencer, "#username").await?;
    dockerhub_login::click_element(&mut ws, &session_id, &mut sequencer, username_node_id).await?;
    for key in username.split_inclusive(|_| true) {
        dockerhub_login::press_key(&mut ws, &session_id, &mut sequencer, key).await?;
    }
    dockerhub_login::press_key(&mut ws, &session_id, &mut sequencer, "\r").await?;

    let password_node_id =
        dockerhub_login::find_element(&mut ws, &session_id, &mut sequencer, "#password").await?;
    dockerhub_login::click_element(&mut ws, &session_id, &mut sequencer, password_node_id).await?;
    for key in password.split_inclusive(|_| true) {
        dockerhub_login::press_key(&mut ws, &session_id, &mut sequencer, key).await?;
    }

    let _: dockerhub_login::cdp::Response<serde_json::Value> = dockerhub_login::send(
        &mut ws,
        dockerhub_login::cdp::Request {
            id: sequencer.next().unwrap(),
            session_id: Some(&session_id),
            method: "Fetch.enable",
            params: dockerhub_login::fetch::EnableParams {
                patterns: vec![dockerhub_login::fetch::RequestPattern {
                    url_pattern: "https://hub.docker.com/*",
                }],
            },
        },
    )
    .await?;

    dockerhub_login::press_key(&mut ws, &session_id, &mut sequencer, "\r").await?;

    let mut auth_header = None;
    while let Some(item) = ws.next().await {
        match item? {
            tokio_tungstenite::tungstenite::Message::Text(msg) => {
                match serde_json::from_str::<
                    dockerhub_login::cdp::Event<dockerhub_login::fetch::RequestPaused>,
                >(&msg)
                {
                    Ok(evt) => {
                        if evt.method == "Fetch.requestPaused" && evt.session_id == session_id {
                            let mut headers = http::HeaderMap::from_iter(
                                evt.params.request.headers.into_iter().filter_map(|(k, v)| {
                                    http::header::HeaderName::from_bytes(k.as_bytes())
                                        .ok()
                                        .map(|k| (k, v))
                                }),
                            );
                            if let Some(auth) = headers.remove(http::header::AUTHORIZATION) {
                                auth_header = Some(auth);
                                break;
                            }
                            let _: dockerhub_login::cdp::Response<serde_json::Value> =
                                dockerhub_login::send(
                                    &mut ws,
                                    dockerhub_login::cdp::Request {
                                        id: sequencer.next().unwrap(),
                                        session_id: Some(&session_id),
                                        method: "Fetch.continueRequest",
                                        params: dockerhub_login::fetch::ContinueRequestParams {
                                            request_id: &evt.params.request_id,
                                        },
                                    },
                                )
                                .await?;
                        }
                    }
                    Err(e) => {
                        tracing::debug!("ignore unknown event ({:?}) {}", e, msg);
                    }
                }
            }
            msg => tracing::warn!(message = ?msg, "non-text WebSocket message was received"),
        }
    }

    let auth_header = auth_header.unwrap();

    ws.send(tokio_tungstenite::tungstenite::Message::Text(
        serde_json::to_string(&dockerhub_login::cdp::Request {
            id: sequencer.next().unwrap(),
            session_id: None,
            method: "Browser.close",
            params: (),
        })?
        .into(),
    ))
    .await?;

    ws.close(None).await?;

    child.wait().await?;

    Ok(auth_header)
}
