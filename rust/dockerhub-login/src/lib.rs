pub mod browser;
pub mod cdp;
pub mod dom;
pub mod fetch;
pub mod input;
pub mod page;
pub mod target;

pub async fn send<'a, S, T, U>(
    ws: &mut S,
    req: crate::cdp::Request<'a, T>,
) -> anyhow::Result<crate::cdp::Response<U>>
where
    S: futures_util::Sink<tokio_tungstenite::tungstenite::Message>
        + futures_util::Stream<
            Item = Result<
                tokio_tungstenite::tungstenite::Message,
                tokio_tungstenite::tungstenite::Error,
            >,
        > + Unpin,
    S::Error: std::error::Error + Send + Sync + 'static,
    T: serde::Serialize,
    U: serde::de::DeserializeOwned,
{
    use futures_util::SinkExt as _;
    ws.send(tokio_tungstenite::tungstenite::Message::Text(
        serde_json::to_string(&req)?.into(),
    ))
    .await?;

    use futures_util::StreamExt as _;
    while let Some(item) = ws.next().await {
        match item? {
            tokio_tungstenite::tungstenite::Message::Text(msg) => {
                match serde_json::from_str::<crate::cdp::Response<U>>(&msg) {
                    Ok(resp) => {
                        if resp.id == req.id && resp.session_id.as_ref() == req.session_id {
                            return Ok(resp);
                        }
                        tracing::debug!(message = ?msg, %req.id, %resp.id, ?req.session_id, ?resp.session_id, "unknown WebSocket message was received")
                    }
                    Err(_) => match serde_json::from_str::<crate::cdp::ErrorResponse>(&msg) {
                        Ok(resp)
                            if resp.id == req.id && resp.session_id.as_ref() == req.session_id =>
                        {
                            return Err(anyhow::Error::from(resp.error));
                        }
                        _ => {}
                    },
                }
            }
            msg => tracing::warn!(message = ?msg, "non-text WebSocket message was received"),
        }
    }
    anyhow::bail!("unexpected EOF of WebSocket");
}

pub async fn find_element<S, I>(
    ws: &mut S,
    session_id: &crate::cdp::SessionId,
    sequencer: &mut I,
    selector: &str,
) -> anyhow::Result<crate::dom::NodeId>
where
    S: futures_util::Sink<tokio_tungstenite::tungstenite::Message>
        + futures_util::Stream<
            Item = Result<
                tokio_tungstenite::tungstenite::Message,
                tokio_tungstenite::tungstenite::Error,
            >,
        > + Unpin,
    S::Error: std::error::Error + Send + Sync + 'static,
    I: Iterator<Item = u64>,
{
    for _ in 0..5 {
        match try_find_element(ws, session_id, sequencer, selector).await {
            Ok(n) => return Ok(n),
            Err(e) => match e.downcast_ref::<crate::cdp::CdpError>() {
                Some(e) if e.code == -32000 => { /* element not found, retrying */ }
                _ => return Err(e),
            },
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
    anyhow::bail!("Could not find element matching {}", selector);
}

async fn try_find_element<S, I>(
    ws: &mut S,
    session_id: &crate::cdp::SessionId,
    sequencer: &mut I,
    selector: &str,
) -> anyhow::Result<crate::dom::NodeId>
where
    S: futures_util::Sink<tokio_tungstenite::tungstenite::Message>
        + futures_util::Stream<
            Item = Result<
                tokio_tungstenite::tungstenite::Message,
                tokio_tungstenite::tungstenite::Error,
            >,
        > + Unpin,
    S::Error: std::error::Error + Send + Sync + 'static,
    I: Iterator<Item = u64>,
{
    let resp: crate::cdp::Response<crate::dom::GetDocumentResult> = send(
        ws,
        crate::cdp::Request {
            id: sequencer.next().unwrap(),
            session_id: Some(session_id),
            method: "DOM.getDocument",
            params: (),
        },
    )
    .await?;
    let document_root_id = resp.result.root.node_id;

    let resp: crate::cdp::Response<crate::dom::QuerySelectorResult> = send(
        ws,
        crate::cdp::Request {
            id: sequencer.next().unwrap(),
            session_id: Some(session_id),
            method: "DOM.querySelector",
            params: crate::dom::QuerySelectorParams {
                node_id: document_root_id,
                selector,
            },
        },
    )
    .await?;
    let node_id = resp.result.node_id;

    // verify
    let _: crate::cdp::Response<serde_json::Value> = send(
        ws,
        crate::cdp::Request {
            id: sequencer.next().unwrap(),
            session_id: Some(session_id),
            method: "DOM.describeNode",
            params: crate::dom::GetBoxModelParams { node_id },
        },
    )
    .await?;

    Ok(node_id)
}

pub async fn click_element<S, I>(
    ws: &mut S,
    session_id: &crate::cdp::SessionId,
    sequencer: &mut I,
    node_id: crate::dom::NodeId,
) -> anyhow::Result<()>
where
    S: futures_util::Sink<tokio_tungstenite::tungstenite::Message>
        + futures_util::Stream<
            Item = Result<
                tokio_tungstenite::tungstenite::Message,
                tokio_tungstenite::tungstenite::Error,
            >,
        > + Unpin,
    S::Error: std::error::Error + Send + Sync + 'static,
    I: Iterator<Item = u64>,
{
    let resp: crate::cdp::Response<crate::dom::GetBoxModelResult> = send(
        ws,
        crate::cdp::Request {
            id: sequencer.next().unwrap(),
            session_id: Some(session_id),
            method: "DOM.getBoxModel",
            params: crate::dom::GetBoxModelParams { node_id },
        },
    )
    .await?;
    let x = (resp.result.model.content[0] + resp.result.model.content[2]) / 2.0;
    let y = (resp.result.model.content[1] + resp.result.model.content[5]) / 2.0;
    let _: crate::cdp::Response<serde_json::Value> = send(
        ws,
        crate::cdp::Request {
            id: sequencer.next().unwrap(),
            session_id: Some(session_id),
            method: "Input.dispatchMouseEvent",
            params: crate::input::DispatchMouseEventParams {
                type_: "mousePressed",
                x,
                y,
                button: "left",
                click_count: 1,
            },
        },
    )
    .await?;
    let _: crate::cdp::Response<serde_json::Value> = send(
        ws,
        crate::cdp::Request {
            id: sequencer.next().unwrap(),
            session_id: Some(session_id),
            method: "Input.dispatchMouseEvent",
            params: crate::input::DispatchMouseEventParams {
                type_: "mouseReleased",
                x,
                y,
                button: "left",
                click_count: 1,
            },
        },
    )
    .await?;
    Ok(())
}

pub async fn press_key<S, I>(
    ws: &mut S,
    session_id: &crate::cdp::SessionId,
    sequencer: &mut I,
    key: &str,
) -> anyhow::Result<()>
where
    S: futures_util::Sink<tokio_tungstenite::tungstenite::Message>
        + futures_util::Stream<
            Item = Result<
                tokio_tungstenite::tungstenite::Message,
                tokio_tungstenite::tungstenite::Error,
            >,
        > + Unpin,
    S::Error: std::error::Error + Send + Sync + 'static,
    I: Iterator<Item = u64>,
{
    let _: crate::cdp::Response<serde_json::Value> = send(
        ws,
        crate::cdp::Request {
            id: sequencer.next().unwrap(),
            session_id: Some(session_id),
            method: "Input.dispatchKeyEvent",
            params: crate::input::DispatchKeyEventParams {
                type_: "keyDown",
                text: Some(key),
            },
        },
    )
    .await?;
    let _: crate::cdp::Response<serde_json::Value> = send(
        ws,
        crate::cdp::Request {
            id: sequencer.next().unwrap(),
            session_id: Some(session_id),
            method: "Input.dispatchKeyEvent",
            params: crate::input::DispatchKeyEventParams {
                type_: "keyUp",
                text: None,
            },
        },
    )
    .await?;
    Ok(())
}
