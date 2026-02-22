use ehrmantraut_core::ir::{Annotations, Block, DeclKind, FunctionDecl, IrNode, Param, ScopeLevel};

use crate::lower::{JsLowerer, LowerError};

impl JsLowerer {
  pub fn lower_function_declaration(
    &mut self,
    node: &crate::cst::CstNode,
  ) -> Result<IrNode, LowerError> {
    let name = self.child_by_field(node, "name").map(|n| self.node_text(n));

    let params = self.lower_formal_parameters(node);

    let body_node = self.child_by_field(node, "body");
    let body = match body_node {
      Some(b) => self.lower_block(b)?,
      None => Block {
        span: node.span,
        body: Vec::new(),
      },
    };

    let is_async = node.children.iter().any(|c| c.kind == "async");
    let is_generator = node.children.iter().any(|c| c.kind == "*");

    Ok(IrNode::FunctionDecl(FunctionDecl {
      span: node.span,
      name,
      params,
      body,
      annotations: Annotations {
        scope_level: Some(ScopeLevel::Module),
        declaration_kind: Some(DeclKind::Function),
        is_async,
        is_generator,
        ..Default::default()
      },
      is_arrow: false,
    }))
  }

  pub(crate) fn lower_formal_parameters(&mut self, func_node: &crate::cst::CstNode) -> Vec<Param> {
    let params_node = self.child_by_field(func_node, "parameters");
    match params_node {
      Some(p) => p
        .children
        .iter()
        .filter(|c| c.named)
        .filter_map(|c| self.lower_param(c).ok())
        .collect(),
      None => Vec::new(),
    }
  }

  pub(crate) fn lower_param(&mut self, node: &crate::cst::CstNode) -> Result<Param, LowerError> {
    match node.kind.as_str() {
      "object_pattern" | "array_pattern" => {
        let pat = self.lower_pattern(node)?;
        Ok(Param {
          span: node.span,
          name: String::new(),
          pattern: Some(pat),
          default_value: None,
        })
      }
      "assignment_pattern" => {
        let left = self.child_by_field(node, "left");
        let right = self.child_by_field(node, "right");

        let (name, pattern) = match left {
          Some(l) if l.kind == "object_pattern" || l.kind == "array_pattern" => {
            let pat = self.lower_pattern(l)?;
            (String::new(), Some(pat))
          }
          Some(l) => (self.node_text(l), None),
          None => (String::new(), None),
        };

        let default_value = right.map(|r| self.lower_expression(r)).transpose()?;

        Ok(Param {
          span: node.span,
          name,
          pattern,
          default_value,
        })
      }
      "rest_pattern" => {
        let inner = self.first_named_child(node);
        let name = inner.map(|n| self.node_text(n)).unwrap_or_default();
        Ok(Param {
          span: node.span,
          name: format!("...{}", name),
          pattern: None,
          default_value: None,
        })
      }
      _ => Ok(Param {
        span: node.span,
        name: self.node_text(node),
        pattern: None,
        default_value: None,
      }),
    }
  }
}
