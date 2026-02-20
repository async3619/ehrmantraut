use ehrmantraut_core::source::Span;
use serde::Serialize;
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../bindings/")]
#[serde(rename_all = "camelCase")]
pub struct CstNode {
  pub kind: String,
  pub named: bool,
  pub span: Span,
  pub text: Option<String>,
  pub children: Vec<CstNode>,
  pub field_name: Option<String>,
}
