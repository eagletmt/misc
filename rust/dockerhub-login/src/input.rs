#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DispatchMouseEventParams {
    #[serde(rename = "type")]
    pub type_: &'static str,
    pub x: f64,
    pub y: f64,
    pub button: &'static str,
    pub click_count: i64,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DispatchKeyEventParams<'a> {
    #[serde(rename = "type")]
    pub type_: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<&'a str>,
}
