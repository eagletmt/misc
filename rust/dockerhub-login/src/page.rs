#[derive(Debug, serde::Serialize)]
pub struct NavigateParams<'a> {
    pub url: &'a str,
}
