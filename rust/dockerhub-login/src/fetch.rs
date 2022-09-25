#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnableParams<'a> {
    pub patterns: Vec<RequestPattern<'a>>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestPattern<'a> {
    pub url_pattern: &'a str,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestPaused {
    pub request_id: RequestId,
    pub request: NetworkRequest,
}

pub type RequestId = String;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkRequest {
    pub headers: std::collections::HashMap<String, String>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContinueRequestParams<'a> {
    pub request_id: &'a RequestId,
}
