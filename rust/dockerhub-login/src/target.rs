pub type TargetId = String;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetTargetsResult {
    pub target_infos: Vec<TargetInfo>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetInfo {
    #[serde(rename = "type")]
    pub type_: String,
    pub target_id: TargetId,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachToTargetParams<'a> {
    pub target_id: &'a TargetId,
    pub flatten: bool,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachToTargetResult {
    pub session_id: crate::cdp::SessionId,
}
