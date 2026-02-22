use ehrmantraut_core::ir::{Call, IrExpr, NewExpr, OpaqueExpr, TaggedTemplate};

use crate::lower::{JsLowerer, LowerError};

impl JsLowerer {
  pub fn lower_call_expression(
    &mut self,
    node: &crate::cst::CstNode,
  ) -> Result<IrExpr, LowerError> {
    let callee_node = self.child_by_field(node, "function");
    let callee = match callee_node {
      Some(c) => self.lower_expression(c)?,
      None => IrExpr::Opaque(OpaqueExpr {
        span: node.span,
        cst_kind: "missing_callee".to_string(),
        text: String::new(),
      }),
    };

    let args_node = self.child_by_field(node, "arguments");

    // Tagged template: tree-sitter represents `tag`hello`` as a call_expression
    // where the arguments field is a template_string instead of an arguments node.
    if let Some(args) = args_node {
      if args.kind == "template_string" {
        let quasi = self.lower_template_as_literal(args)?;
        return Ok(IrExpr::TaggedTemplate(TaggedTemplate {
          span: node.span,
          tag: Box::new(callee),
          quasi: Box::new(quasi),
        }));
      }
    }

    let arguments = match args_node {
      Some(args) => args
        .children
        .iter()
        .filter(|c| c.named)
        .map(|c| self.lower_expression(c))
        .collect::<Result<Vec<_>, _>>()?,
      None => Vec::new(),
    };

    Ok(IrExpr::Call(Call {
      span: node.span,
      callee: Box::new(callee),
      arguments,
      optional: Self::has_optional_chain(node),
    }))
  }

  pub fn lower_new_expression(&mut self, node: &crate::cst::CstNode) -> Result<IrExpr, LowerError> {
    let callee_node = self.child_by_field(node, "constructor");
    let callee = match callee_node {
      Some(c) => self.lower_expression(c)?,
      None => IrExpr::Opaque(OpaqueExpr {
        span: node.span,
        cst_kind: "missing_constructor".to_string(),
        text: String::new(),
      }),
    };

    let args_node = self.child_by_field(node, "arguments");
    let arguments = match args_node {
      Some(args) => args
        .children
        .iter()
        .filter(|c| c.named)
        .map(|c| self.lower_expression(c))
        .collect::<Result<Vec<_>, _>>()?,
      None => Vec::new(),
    };

    Ok(IrExpr::NewExpr(NewExpr {
      span: node.span,
      callee: Box::new(callee),
      arguments,
    }))
  }
}
