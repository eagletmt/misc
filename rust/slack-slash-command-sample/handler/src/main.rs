#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiGatewayV2Request {
    body: Option<String>,
    headers: std::collections::HashMap<String, String>,
    is_base64_encoded: bool,
    request_context: RequestContext,
}

#[derive(Debug, serde::Deserialize)]
struct RequestContext {
    http: RequestContextHttp,
}

#[derive(Debug, serde::Deserialize)]
struct RequestContextHttp {
    path: String,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ApiGatewayV2Response {
    #[serde(skip_serializing_if = "Option::is_none")]
    cookies: Option<Vec<String>>,
    is_base64_encoded: bool,
    status_code: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    headers: Option<std::collections::HashMap<String, String>>,
    body: String,
}
impl Default for ApiGatewayV2Response {
    fn default() -> Self {
        Self {
            cookies: None,
            is_base64_encoded: false,
            status_code: 200,
            headers: None,
            body: "".to_owned(),
        }
    }
}

#[netlify_lambda::lambda]
#[tokio::main]
async fn main(
    request: ApiGatewayV2Request,
    _: netlify_lambda::Context,
) -> Result<ApiGatewayV2Response, anyhow::Error> {
    println!("{:?}", request);
    if request.body.is_none() {
        return Ok(ApiGatewayV2Response {
            status_code: 400,
            body: "body is empty".to_owned(),
            ..Default::default()
        });
    };
    let body = request.body.unwrap();
    let body = if request.is_base64_encoded {
        String::from_utf8_lossy(&base64::decode(body)?).into_owned()
    } else {
        body
    };
    println!("decoded body: {}", body);

    if let Err(resp) = verify_signature(&request.headers, &body) {
        return Ok(resp);
    }

    println!("path = '{}'", request.request_context.http.path);
    if request.request_context.http.path == "/slash_command" {
        handle_slash_command(body).await
    } else if request.request_context.http.path == "/interactive" {
        handle_interactive(body).await
    } else if request.request_context.http.path == "/external_select" {
        handle_external_select(body).await
    } else {
        Ok(ApiGatewayV2Response {
            status_code: 404,
            ..Default::default()
        })
    }
}

fn verify_signature(
    headers: &std::collections::HashMap<String, String>,
    body: &str,
) -> Result<(), ApiGatewayV2Response> {
    // Verify signature https://api.slack.com/authentication/verifying-requests-from-slack
    let signing_secret = match std::env::var("SLACK_SIGNING_SECRET") {
        Ok(secret) => secret,
        Err(_) => {
            return Err(ApiGatewayV2Response {
                status_code: 500,
                body: "SLACK_SIGNING_SECRET is missing".to_owned(),
                ..Default::default()
            });
        }
    };
    let slack_signature = match headers.get("x-slack-signature") {
        Some(t) => t,
        None => {
            return Err(ApiGatewayV2Response {
                status_code: 400,
                body: "X-Slack-Signature is missing".to_owned(),
                ..Default::default()
            });
        }
    };
    let timestamp = match headers.get("x-slack-request-timestamp") {
        Some(s) => match s.parse::<i64>() {
            Ok(t) => t,
            Err(_) => {
                return Err(ApiGatewayV2Response {
                    status_code: 400,
                    body: "X-Slack-Request-Timestamp is malformed".to_owned(),
                    ..Default::default()
                });
            }
        },
        None => {
            return Err(ApiGatewayV2Response {
                status_code: 400,
                body: "X-Slack-Request-Timestamp is missing".to_owned(),
                ..Default::default()
            });
        }
    };
    let sig_basestring = format!("v0:{}:{}", timestamp, body);
    let hmac_key = ring::hmac::Key::new(ring::hmac::HMAC_SHA256, signing_secret.as_bytes());
    let sig = format!(
        "v0={}",
        hex::encode(ring::hmac::sign(&hmac_key, sig_basestring.as_bytes()))
    );
    if sig != *slack_signature {
        return Err(ApiGatewayV2Response {
            status_code: 400,
            body: format!(
                "X-Slack-Signature mismatch: given={} computed={}",
                slack_signature, sig
            ),
            ..Default::default()
        });
    }
    println!("X-Slack-Signature is verified: {}", slack_signature);
    Ok(())
}

#[derive(Debug, serde::Deserialize)]
struct SlashCommandParams {
    team_id: String,
    team_domain: String,
    channel_id: String,
    channel_name: String,
    user_id: String,
    user_name: String,
    command: String,
    text: String,
    response_url: String,
    trigger_id: String,
}

#[derive(Debug, serde::Serialize)]
struct SlackResponse {
    response_type: SlackResponseType,
    blocks: Vec<Block>,
}

#[derive(Debug, serde::Serialize)]
enum SlackResponseType {
    #[serde(rename = "in_channel")]
    InChannel,
    #[serde(rename = "ephemeral")]
    Ephemeral,
}

#[derive(Debug, serde::Serialize)]
#[serde(tag = "type")]
enum Block {
    #[serde(rename = "section")]
    Section {
        #[serde(skip_serializing_if = "Option::is_none")]
        block_id: Option<String>,
        text: Text,
        #[serde(skip_serializing_if = "Option::is_none")]
        accessory: Option<Accessory>,
    },
}

#[derive(Debug, serde::Serialize)]
#[serde(tag = "type")]
enum Text {
    #[serde(rename = "plain_text")]
    PlainText { text: String },
    #[serde(rename = "mrkdwn")]
    Mrkdwn { text: String },
}

#[derive(Debug, serde::Serialize)]
#[serde(tag = "type")]
enum Accessory {
    #[serde(rename = "external_select")]
    ExternalSelect {
        placeholder: Text,
        action_id: String,
        min_query_length: usize,
    },
}

#[derive(Debug, serde::Serialize)]
struct SlashCommandResponse {
    trigger_id: String,
    view: SlashCommandView,
}

#[derive(Debug, serde::Serialize)]
#[serde(tag = "type")]
enum SlashCommandView {
    #[serde(rename = "modal")]
    Modal(Modal),
}
#[derive(Debug, serde::Serialize)]
struct Modal {
    private_metadata: String,
    title: Text,
    submit: Text,
    close: Text,
    blocks: Vec<Block>,
}

async fn handle_slash_command(body: String) -> Result<ApiGatewayV2Response, anyhow::Error> {
    let params: SlashCommandParams = match serde_urlencoded::from_str(&body) {
        Ok(p) => p,
        Err(e) => {
            return Ok(ApiGatewayV2Response {
                status_code: 400,
                body: format!("Failed to deserialize slash command input: {:?}", e),
                ..Default::default()
            });
        }
    };

    let access_token = match std::env::var("SLACK_ACCESS_TOKEN") {
        Ok(token) => token,
        Err(_) => {
            return Ok(ApiGatewayV2Response {
                status_code: 500,
                body: "SLACK_ACCESS_TOKEN is missing".to_owned(),
                ..Default::default()
            });
        }
    };

    if params.text.is_empty() {
        let client = reqwest::Client::new();
        let resp = client
            .post("https://slack.com/api/views.open")
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .bearer_auth(access_token)
            .json(&SlashCommandResponse {
                trigger_id: params.trigger_id,
                view: SlashCommandView::Modal(Modal {
                    private_metadata: serde_json::to_string(&PrivateMetadata {
                        response_url: params.response_url,
                    })?,
                    title: Text::PlainText {
                        text: "Deploy bot".to_owned(),
                    },
                    submit: Text::PlainText {
                        text: "Deploy".to_owned(),
                    },
                    close: Text::PlainText {
                        text: "Cancel".to_owned(),
                    },
                    blocks: vec![Block::Section {
                        block_id: Some("deploy_section".to_owned()),
                        text: Text::PlainText {
                            text: "Choose deploy target".to_owned(),
                        },
                        accessory: Some(Accessory::ExternalSelect {
                            placeholder: Text::PlainText {
                                text: "Placeholder".to_owned(),
                            },
                            action_id: "app_id".to_owned(),
                            min_query_length: 1,
                        }),
                    }],
                }),
            })
            .send()
            .await?;
        let b = resp.text().await;
        println!("views.open: {:?}", b);
        Ok(ApiGatewayV2Response {
            ..Default::default()
        })
    } else if params.text == "help" {
        let response = serde_json::to_string(&SlackResponse {
            response_type: SlackResponseType::Ephemeral,
            blocks: vec![
                Block::Section {
                    block_id: None,
                    accessory: None,
                    text: Text::Mrkdwn {
                        text: format!("`{}` command deploys Hako app", params.command),
                    },
                },
                Block::Section {
                    block_id: None,
                    accessory: None,
                    text: Text::Mrkdwn {
                        text: format!("- `{command}`: Choose deploy target from list.\n- `{command} help`: Display this message.\n- `{command} <app_id>`: Deploy `<app_id>`.", command=params.command),
                    },
                },
            ],
        })?;
        println!("response: {}", response);
        Ok(ApiGatewayV2Response {
            headers: Some(
                [("content-type".to_owned(), "application/json".to_owned())]
                    .iter()
                    .cloned()
                    .collect(),
            ),
            body: response,
            ..Default::default()
        })
    } else {
        Ok(ApiGatewayV2Response {
            headers: Some(
                [("content-type".to_owned(), "application/json".to_owned())]
                    .iter()
                    .cloned()
                    .collect(),
            ),
            body: serde_json::to_string(&SlackResponse {
                response_type: SlackResponseType::InChannel,
                blocks: vec![Block::Section {
                    block_id: None,
                    accessory: None,
                    text: Text::Mrkdwn {
                        text: format!("<@{}> started to deploy {}", params.user_id, params.text),
                    },
                }],
            })?,
            ..Default::default()
        })
    }
}

#[derive(Debug, serde::Deserialize)]
struct PayloadParams {
    payload: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct PrivateMetadata {
    response_url: String,
}

#[derive(Debug, serde::Deserialize)]
struct InteractivePayload {
    #[serde(rename = "type")]
    type_: InteractivePayloadType,
    user: InteractivePayloadUser,
    view: InteractivePayloadView,
}
#[derive(Debug, serde::Deserialize, PartialEq)]
enum InteractivePayloadType {
    #[serde(rename = "block_actions")]
    BlockActions,
    #[serde(rename = "view_submission")]
    ViewSubmission,
}
#[derive(Debug, serde::Deserialize)]
struct InteractivePayloadUser {
    id: String,
    username: String,
}
#[derive(Debug, serde::Deserialize)]
struct InteractivePayloadView {
    state: InteractivePayloadViewState,
    private_metadata: String,
}
#[derive(Debug, serde::Deserialize)]
struct InteractivePayloadViewState {
    values: std::collections::HashMap<
        String,
        std::collections::HashMap<String, InteractivePayloadActionState>,
    >,
}
#[derive(Debug, serde::Deserialize)]
#[serde(tag = "type")]
enum InteractivePayloadActionState {
    #[serde(rename = "external_select")]
    ExternalSelect { selected_option: SelectedOption },
}
#[derive(Debug, serde::Deserialize)]
struct SelectedOption {
    value: String,
}

async fn handle_interactive(body: String) -> Result<ApiGatewayV2Response, anyhow::Error> {
    let params: PayloadParams = match serde_urlencoded::from_str(&body) {
        Ok(p) => p,
        Err(e) => {
            return Ok(ApiGatewayV2Response {
                status_code: 400,
                body: format!("Failed to deserialize interactive payload: {:?}", e),
                ..Default::default()
            });
        }
    };
    println!("payload: {}", params.payload);
    let payload: InteractivePayload = match serde_json::from_str(&params.payload) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("failed to parse payload: {:?}", e);
            return Ok(ApiGatewayV2Response {
                status_code: 400,
                body: format!("Failed to deserialize interactive payload: {:?}", e),
                ..Default::default()
            });
        }
    };
    if payload.type_ != InteractivePayloadType::ViewSubmission {
        return Ok(ApiGatewayV2Response {
            ..Default::default()
        });
    }

    let private_metadata: PrivateMetadata = serde_json::from_str(&payload.view.private_metadata)?;
    let app_id = match payload
        .view
        .state
        .values
        .get("deploy_section")
        .unwrap()
        .get("app_id")
        .unwrap()
    {
        InteractivePayloadActionState::ExternalSelect { selected_option } => &selected_option.value,
    };
    let client = reqwest::Client::new();
    client
        .post(&private_metadata.response_url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .json(&SlackResponse {
            response_type: SlackResponseType::InChannel,
            blocks: vec![Block::Section {
                block_id: None,
                accessory: None,
                text: Text::Mrkdwn {
                    text: format!("<@{}> started to deploy {}", payload.user.id, app_id),
                },
            }],
        })
        .send()
        .await?;
    Ok(ApiGatewayV2Response {
        ..Default::default()
    })
}

#[derive(Debug, serde::Deserialize)]
struct ExternalSelectPayload {
    #[serde(rename = "type")]
    type_: String,
    value: String,
}
#[derive(Debug, serde::Serialize)]
struct ExternalSelectResponse {
    options: Vec<OptionResponse>,
}
#[derive(Debug, serde::Serialize)]
struct OptionResponse {
    text: Text,
    value: String,
}

async fn handle_external_select(body: String) -> Result<ApiGatewayV2Response, anyhow::Error> {
    let params: PayloadParams = match serde_urlencoded::from_str(&body) {
        Ok(p) => p,
        Err(e) => {
            return Ok(ApiGatewayV2Response {
                status_code: 400,
                body: format!("Failed to deserialize interactive payload: {:?}", e),
                ..Default::default()
            });
        }
    };
    println!("payload: {}", params.payload);
    let payload: ExternalSelectPayload = match serde_json::from_str(&params.payload) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("failed to parse payload: {:?}", e);
            return Ok(ApiGatewayV2Response {
                status_code: 400,
                body: format!("Failed to deserialize interactive payload: {:?}", e),
                ..Default::default()
            });
        }
    };
    let mut options = Vec::new();
    for i in 0..200 {
        let value = format!("foo-{}", i);
        if value.contains(&payload.value) {
            options.push(OptionResponse {
                text: Text::PlainText {
                    text: value.clone(),
                },
                value,
            });
            const MAX_OPTIONS_LEN: usize = 100;
            if options.len() >= MAX_OPTIONS_LEN {
                break;
            }
        }
    }
    let response = serde_json::to_string(&ExternalSelectResponse { options })?;
    Ok(ApiGatewayV2Response {
        headers: Some(
            [("content-type".to_owned(), "application/json".to_owned())]
                .iter()
                .cloned()
                .collect(),
        ),
        body: response,
        ..Default::default()
    })
}
