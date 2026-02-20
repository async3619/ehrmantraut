use ehrmantraut_core::source::Span;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CstNode {
  pub kind: String,
  pub named: bool,
  pub span: Span,
  pub text: Option<String>,
  pub children: Vec<CstNode>,
  pub field_name: Option<String>,
}
