use ehrmantraut_core::ir::{DeclKind, ScopeLevel};

use crate::cst::CstNode;

use super::JsLowerer;

impl JsLowerer {
  pub(crate) fn first_named_child<'a>(&self, node: &'a CstNode) -> Option<&'a CstNode> {
    node.children.iter().find(|c| c.named)
  }

  pub(crate) fn child_by_field<'a>(&self, node: &'a CstNode, field: &str) -> Option<&'a CstNode> {
    node
      .children
      .iter()
      .find(|c| c.field_name.as_deref() == Some(field))
  }

  pub(crate) fn node_text(&self, node: &CstNode) -> String {
    if let Some(text) = &node.text {
      return text.clone();
    }
    // For branch nodes, reconstruct from source if available
    if !self.source.is_empty() {
      let start = node.span.start.offset as usize;
      let end = node.span.end.offset as usize;
      if start <= end && end <= self.source.len() {
        return self.source[start..end].to_string();
      }
    }
    // Fallback: concatenate leaf text
    self.collect_text(node)
  }

  fn collect_text(&self, node: &CstNode) -> String {
    if let Some(text) = &node.text {
      return text.clone();
    }
    node
      .children
      .iter()
      .map(|c| self.collect_text(c))
      .collect::<Vec<_>>()
      .join("")
  }

  pub(crate) fn strip_quotes(text: String) -> String {
    if text.len() >= 2
      && ((text.starts_with('"') && text.ends_with('"'))
        || (text.starts_with('\'') && text.ends_with('\''))
        || (text.starts_with('`') && text.ends_with('`')))
    {
      text[1..text.len() - 1].to_string()
    } else {
      text
    }
  }

  pub(crate) fn decl_kind_and_scope(kind_str: &str) -> (Option<DeclKind>, Option<ScopeLevel>) {
    match kind_str {
      "let" => (Some(DeclKind::Let), Some(ScopeLevel::Block)),
      "const" => (Some(DeclKind::Const), Some(ScopeLevel::Block)),
      "var" => (Some(DeclKind::Var), Some(ScopeLevel::Function)),
      _ => (None, None),
    }
  }

  pub(crate) fn is_ts_type_node(kind: &str) -> bool {
    matches!(
      kind,
      "type_annotation"
        | "type_parameters"
        | "type_arguments"
        | "interface_declaration"
        | "type_alias_declaration"
        | "enum_declaration"
        | "as_expression"
        | "satisfies_expression"
        | "non_null_expression"
        | "type_assertion"
    )
  }

  pub(crate) fn has_optional_chain(node: &CstNode) -> bool {
    node
      .children
      .iter()
      .any(|c| c.kind == "optional_chain" || c.field_name.as_deref() == Some("optional_chain"))
  }
}
