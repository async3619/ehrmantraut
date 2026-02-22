use ehrmantraut_core::ir::{Annotations, IrNode, VariableDecl};

use crate::lower::{JsLowerer, LowerError};

impl JsLowerer {
  pub fn lower_variable_declaration(
    &mut self,
    node: &crate::cst::CstNode,
  ) -> Result<Vec<IrNode>, LowerError> {
    let kind_text = node
      .children
      .iter()
      .find(|c| !c.named && matches!(c.kind.as_str(), "let" | "const" | "var"))
      .map(|c| c.kind.as_str())
      .unwrap_or("");

    let (decl_kind, scope_level) = Self::decl_kind_and_scope(kind_text);

    let declarators: Vec<_> = node
      .children
      .iter()
      .filter(|c| c.kind == "variable_declarator")
      .collect();

    let mut nodes = Vec::new();
    for declarator in &declarators {
      let name_node = self.child_by_field(declarator, "name");

      let (name, pattern) = match name_node {
        Some(n) if n.kind == "object_pattern" || n.kind == "array_pattern" => {
          let pat = self.lower_pattern(n)?;
          (String::new(), Some(pat))
        }
        Some(n) => (self.node_text(n), None),
        None => (String::new(), None),
      };

      let value = self
        .child_by_field(declarator, "value")
        .map(|v| self.lower_expression(v))
        .transpose()?;

      nodes.push(IrNode::VariableDecl(VariableDecl {
        span: declarator.span,
        name,
        value,
        pattern,
        annotations: Annotations {
          scope_level: scope_level.clone(),
          declaration_kind: decl_kind.clone(),
          ..Default::default()
        },
      }));
    }

    if nodes.is_empty() {
      nodes.push(IrNode::VariableDecl(VariableDecl {
        span: node.span,
        name: String::new(),
        value: None,
        pattern: None,
        annotations: Annotations {
          scope_level,
          declaration_kind: decl_kind,
          ..Default::default()
        },
      }));
    }

    Ok(nodes)
  }
}
