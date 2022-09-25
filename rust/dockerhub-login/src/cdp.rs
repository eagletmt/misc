pub type SessionId = String;

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Request<'a, T> {
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<&'a SessionId>,
    pub method: &'static str,
    pub params: T,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response<T> {
    pub id: u64,
    pub session_id: Option<SessionId>,
    pub result: T,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    pub id: u64,
    pub session_id: Option<SessionId>,
    pub error: CdpError,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CdpError {
    pub code: i64,
    pub message: String,
}
impl std::fmt::Display for CdpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CdpError: code={} message={}", self.code, self.message)
    }
}
impl std::error::Error for CdpError {}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event<T> {
    pub session_id: String,
    pub method: String,
    pub params: T,
}
