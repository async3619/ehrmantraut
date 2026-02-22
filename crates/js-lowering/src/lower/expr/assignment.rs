use ehrmantraut_core::ir::{Assignment, IrExpr, OpaqueExpr};

use crate::lower::{JsLowerer, LowerError};

impl JsLowerer {
  pub fn lower_assignment_expression(
    &mut self,
    node: &crate::cst::CstNode,
  ) -> Result<IrExpr, LowerError> {
    let left = self.child_by_field(node, "left");
    let right = self.child_by_field(node, "right");

    // Find the operator token
    let operator = node
      .children
      .iter()
      .find(|c| {
        !c.named
          && matches!(
            c.kind.as_str(),
            "="
              | "+="
              | "-="
              | "*="
              | "/="
              | "%="
              | "**="
              | "<<="
              | ">>="
              | ">>>="
              | "&="
              | "|="
              | "^="
              | "&&="
              | "||="
              | "??="
          )
      })
      .map(|c| c.kind.clone())
      .unwrap_or_else(|| "=".to_string());

    let target = match left {
      Some(l) => self.lower_expression(l)?,
      None => IrExpr::Opaque(OpaqueExpr {
        span: node.span,
        cst_kind: "missing_left".to_string(),
        text: String::new(),
      }),
    };

    let value = match right {
      Some(r) => self.lower_expression(r)?,
      None => IrExpr::Opaque(OpaqueExpr {
        span: node.span,
        cst_kind: "missing_right".to_string(),
        text: String::new(),
      }),
    };

    Ok(IrExpr::Assignment(Assignment {
      span: node.span,
      target: Box::new(target),
      operator,
      value: Box::new(value),
    }))
  }
}
