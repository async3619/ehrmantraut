use ehrmantraut_core::ir::{
  BinaryExpr, ConditionalExpr, IrExpr, OpaqueExpr, UnaryExpr, UpdateExpr,
};

use crate::lower::{JsLowerer, LowerError};

impl JsLowerer {
  pub fn lower_binary_expression(
    &mut self,
    node: &crate::cst::CstNode,
  ) -> Result<IrExpr, LowerError> {
    let left = self.child_by_field(node, "left");
    let right = self.child_by_field(node, "right");
    let operator = self.child_by_field(node, "operator");

    let op_str = operator.map(|o| self.node_text(o)).unwrap_or_default();

    let left_expr = match left {
      Some(l) => self.lower_expression(l)?,
      None => IrExpr::Opaque(OpaqueExpr {
        span: node.span,
        cst_kind: "missing_left".to_string(),
        text: String::new(),
      }),
    };

    let right_expr = match right {
      Some(r) => self.lower_expression(r)?,
      None => IrExpr::Opaque(OpaqueExpr {
        span: node.span,
        cst_kind: "missing_right".to_string(),
        text: String::new(),
      }),
    };

    Ok(IrExpr::BinaryExpr(BinaryExpr {
      span: node.span,
      left: Box::new(left_expr),
      operator: op_str,
      right: Box::new(right_expr),
    }))
  }

  pub fn lower_unary_expression(
    &mut self,
    node: &crate::cst::CstNode,
  ) -> Result<IrExpr, LowerError> {
    let operator = self.child_by_field(node, "operator");
    let operand = self.child_by_field(node, "argument");

    let op_str = operator.map(|o| self.node_text(o)).unwrap_or_default();

    let operand_expr = match operand {
      Some(a) => self.lower_expression(a)?,
      None => IrExpr::Opaque(OpaqueExpr {
        span: node.span,
        cst_kind: "missing_operand".to_string(),
        text: String::new(),
      }),
    };

    Ok(IrExpr::UnaryExpr(UnaryExpr {
      span: node.span,
      operator: op_str,
      operand: Box::new(operand_expr),
      prefix: true,
    }))
  }

  pub fn lower_update_expression(
    &mut self,
    node: &crate::cst::CstNode,
  ) -> Result<IrExpr, LowerError> {
    let operand = self.child_by_field(node, "argument");
    let operator = self.child_by_field(node, "operator");

    let op_str = operator.map(|o| self.node_text(o)).unwrap_or_default();

    let operand_expr = match operand {
      Some(a) => self.lower_expression(a)?,
      None => IrExpr::Opaque(OpaqueExpr {
        span: node.span,
        cst_kind: "missing_operand".to_string(),
        text: String::new(),
      }),
    };

    // Determine prefix vs postfix by comparing span offsets of operator and operand.
    // If either is missing (malformed CST), default to prefix as a reasonable fallback
    // since the exact form cannot be determined and the operand is already lowered as OpaqueExpr.
    let prefix = match (operator, operand) {
      (Some(op), Some(arg)) => op.span.start.offset < arg.span.start.offset,
      _ => true,
    };

    Ok(IrExpr::UpdateExpr(UpdateExpr {
      span: node.span,
      operator: op_str,
      operand: Box::new(operand_expr),
      prefix,
    }))
  }

  pub fn lower_conditional_expression(
    &mut self,
    node: &crate::cst::CstNode,
  ) -> Result<IrExpr, LowerError> {
    let condition = self.child_by_field(node, "condition");
    let consequent = self.child_by_field(node, "consequence");
    let alternate = self.child_by_field(node, "alternative");

    let cond_expr = match condition {
      Some(c) => self.lower_expression(c)?,
      None => IrExpr::Opaque(OpaqueExpr {
        span: node.span,
        cst_kind: "missing_condition".to_string(),
        text: String::new(),
      }),
    };

    let cons_expr = match consequent {
      Some(c) => self.lower_expression(c)?,
      None => IrExpr::Opaque(OpaqueExpr {
        span: node.span,
        cst_kind: "missing_consequent".to_string(),
        text: String::new(),
      }),
    };

    let alt_expr = match alternate {
      Some(a) => self.lower_expression(a)?,
      None => IrExpr::Opaque(OpaqueExpr {
        span: node.span,
        cst_kind: "missing_alternate".to_string(),
        text: String::new(),
      }),
    };

    Ok(IrExpr::ConditionalExpr(ConditionalExpr {
      span: node.span,
      condition: Box::new(cond_expr),
      consequent: Box::new(cons_expr),
      alternate: Box::new(alt_expr),
    }))
  }
}
