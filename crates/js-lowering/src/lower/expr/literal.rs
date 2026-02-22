use ehrmantraut_core::ir::{Identifier, IrExpr, Literal, LiteralValue, TemplateLiteral};

use crate::lower::{JsLowerer, LowerError};

impl JsLowerer {
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
    let inner = Self::strip_quotes(text);
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

  pub fn lower_template_string(
    &mut self,
    node: &crate::cst::CstNode,
  ) -> Result<IrExpr, LowerError> {
    let has_interpolation = node
      .children
      .iter()
      .any(|c| c.kind == "template_substitution");

    if !has_interpolation {
      // Simple template string without interpolation — lower as string literal
      return Ok(self.lower_string_literal(node));
    }

    let mut quasis = Vec::new();
    let mut expressions = Vec::new();
    let mut current_quasi = String::new();

    for child in &node.children {
      match child.kind.as_str() {
        "string_fragment" | "escape_sequence" => {
          current_quasi.push_str(&self.node_text(child));
        }
        "template_substitution" => {
          quasis.push(std::mem::take(&mut current_quasi));
          if let Some(expr_node) = self.first_named_child(child) {
            let expr = self.lower_expression(expr_node)?;
            expressions.push(expr);
          }
        }
        _ => {}
      }
    }
    // Push the trailing quasi
    quasis.push(current_quasi);

    debug_assert_eq!(
      quasis.len(),
      expressions.len() + 1,
      "TemplateLiteral invariant: quasis.len() must equal expressions.len() + 1"
    );

    Ok(IrExpr::TemplateLiteral(TemplateLiteral {
      span: node.span,
      quasis,
      expressions,
    }))
  }

  /// Lower a template string always as `TemplateLiteral`, even without interpolation.
  /// Used for tagged template quasi to preserve template-literal structure in the IR.
  pub fn lower_template_as_literal(
    &mut self,
    node: &crate::cst::CstNode,
  ) -> Result<IrExpr, LowerError> {
    let mut quasis = Vec::new();
    let mut expressions = Vec::new();
    let mut current_quasi = String::new();

    for child in &node.children {
      match child.kind.as_str() {
        "string_fragment" | "escape_sequence" => {
          current_quasi.push_str(&self.node_text(child));
        }
        "template_substitution" => {
          quasis.push(std::mem::take(&mut current_quasi));
          if let Some(expr_node) = self.first_named_child(child) {
            let expr = self.lower_expression(expr_node)?;
            expressions.push(expr);
          }
        }
        _ => {}
      }
    }
    quasis.push(current_quasi);

    debug_assert_eq!(
      quasis.len(),
      expressions.len() + 1,
      "TemplateLiteral invariant: quasis.len() must equal expressions.len() + 1"
    );

    Ok(IrExpr::TemplateLiteral(TemplateLiteral {
      span: node.span,
      quasis,
      expressions,
    }))
  }
}
