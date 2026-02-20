use ehrmantraut_core::ir::{
  Assignment, BinaryExpr, Call, Identifier, IrExpr, Literal, LiteralValue, MemberAccess, OpaqueExpr,
};

use super::{JsLowerer, LowerError};

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
    }))
  }

  pub fn lower_member_expression(
    &mut self,
    node: &crate::cst::CstNode,
  ) -> Result<IrExpr, LowerError> {
    let object = self.child_by_field(node, "object");
    let property = self.child_by_field(node, "property");

    let obj_expr = match object {
      Some(o) => self.lower_expression(o)?,
      None => IrExpr::Opaque(OpaqueExpr {
        span: node.span,
        cst_kind: "missing_object".to_string(),
        text: String::new(),
      }),
    };

    let prop_expr = match property {
      Some(p) => self.lower_expression(p)?,
      None => IrExpr::Opaque(OpaqueExpr {
        span: node.span,
        cst_kind: "missing_property".to_string(),
        text: String::new(),
      }),
    };

    Ok(IrExpr::MemberAccess(MemberAccess {
      span: node.span,
      object: Box::new(obj_expr),
      property: Box::new(prop_expr),
      computed: false,
    }))
  }

  pub fn lower_subscript_expression(
    &mut self,
    node: &crate::cst::CstNode,
  ) -> Result<IrExpr, LowerError> {
    let object = self.child_by_field(node, "object");
    let index = self.child_by_field(node, "index");

    let obj_expr = match object {
      Some(o) => self.lower_expression(o)?,
      None => IrExpr::Opaque(OpaqueExpr {
        span: node.span,
        cst_kind: "missing_object".to_string(),
        text: String::new(),
      }),
    };

    let idx_expr = match index {
      Some(i) => self.lower_expression(i)?,
      None => IrExpr::Opaque(OpaqueExpr {
        span: node.span,
        cst_kind: "missing_index".to_string(),
        text: String::new(),
      }),
    };

    Ok(IrExpr::MemberAccess(MemberAccess {
      span: node.span,
      object: Box::new(obj_expr),
      property: Box::new(idx_expr),
      computed: true,
    }))
  }

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

  pub fn lower_identifier(&self, node: &crate::cst::CstNode) -> IrExpr {
    IrExpr::Identifier(Identifier {
      span: node.span,
      name: self.node_text(node),
    })
  }

  pub fn lower_number_literal(&self, node: &crate::cst::CstNode) -> IrExpr {
    let text = self.node_text(node);
    let value = text.parse::<f64>().unwrap_or(f64::NAN);
    IrExpr::Literal(Literal {
      span: node.span,
      value: LiteralValue::Number(value),
    })
  }

  pub fn lower_string_literal(&self, node: &crate::cst::CstNode) -> IrExpr {
    let text = self.node_text(node);
    // Strip surrounding quotes
    let inner = if (text.starts_with('"') && text.ends_with('"'))
      || (text.starts_with('\'') && text.ends_with('\''))
      || (text.starts_with('`') && text.ends_with('`'))
    {
      text[1..text.len() - 1].to_string()
    } else {
      text
    };
    IrExpr::Literal(Literal {
      span: node.span,
      value: LiteralValue::String(inner),
    })
  }

  pub fn lower_boolean_literal(&self, node: &crate::cst::CstNode) -> IrExpr {
    let value = node.kind == "true";
    IrExpr::Literal(Literal {
      span: node.span,
      value: LiteralValue::Boolean(value),
    })
  }

  pub fn lower_null_literal(&self, node: &crate::cst::CstNode) -> IrExpr {
    IrExpr::Literal(Literal {
      span: node.span,
      value: LiteralValue::Null,
    })
  }

  pub fn lower_undefined(&self, node: &crate::cst::CstNode) -> IrExpr {
    IrExpr::Literal(Literal {
      span: node.span,
      value: LiteralValue::Undefined,
    })
  }
}
