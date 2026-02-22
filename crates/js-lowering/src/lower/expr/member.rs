use ehrmantraut_core::ir::{IrExpr, MemberAccess, OpaqueExpr};

use crate::lower::{JsLowerer, LowerError};

impl JsLowerer {
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
      optional: Self::has_optional_chain(node),
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
      optional: Self::has_optional_chain(node),
    }))
  }
}
