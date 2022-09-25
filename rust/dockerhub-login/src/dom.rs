#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetDocumentResult {
    pub root: Node,
}
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Node {
    pub node_id: NodeId,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuerySelectorParams<'a> {
    pub node_id: NodeId,
    pub selector: &'a str,
}
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuerySelectorResult {
    pub node_id: NodeId,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetBoxModelParams {
    pub node_id: NodeId,
}
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetBoxModelResult {
    pub model: BoxModel,
}
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoxModel {
    pub content: Quad,
}

pub type Quad = [f64; 8];
pub type NodeId = i64;
