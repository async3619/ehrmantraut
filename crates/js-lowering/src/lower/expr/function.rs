use ehrmantraut_core::ir::{
  Annotations, AwaitExpr, Block, FunctionDecl, IrExpr, IrNode, OpaqueExpr, Param, ReturnStmt,
  SpreadExpr, YieldExpr,
};

use crate::lower::{JsLowerer, LowerError};

impl JsLowerer {
  pub fn lower_function_expression(
    &mut self,
    node: &crate::cst::CstNode,
  ) -> Result<IrExpr, LowerError> {
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

    Ok(IrExpr::FunctionExpr(FunctionDecl {
      span: node.span,
      name,
      params,
      body,
      annotations: Annotations {
        is_async,
        is_generator,
        ..Default::default()
      },
      is_arrow: false,
    }))
  }

  pub fn lower_arrow_function(&mut self, node: &crate::cst::CstNode) -> Result<IrExpr, LowerError> {
    let is_async = node.children.iter().any(|c| c.kind == "async");

    // Parameters: may be a single identifier or formal_parameters
    let params = self
      .child_by_field(node, "parameters")
      .or_else(|| self.child_by_field(node, "parameter"))
      .map(|p| {
        if p.kind == "identifier" {
          vec![Param {
            span: p.span,
            name: self.node_text(p),
            pattern: None,
            default_value: None,
          }]
        } else {
          p.children
            .iter()
            .filter(|c| c.named && !Self::is_ts_type_node(&c.kind))
            .filter_map(|c| self.lower_param(c).ok())
            .collect()
        }
      })
      .unwrap_or_default();

    let body_node = self.child_by_field(node, "body");
    let body = match body_node {
      Some(b) if b.kind == "statement_block" => self.lower_block(b)?,
      Some(b) => {
        // Concise body: wrap expression in implicit return
        let expr = self.lower_expression(b)?;
        Block {
          span: b.span,
          body: vec![IrNode::Return(ReturnStmt {
            span: b.span,
            value: Some(expr),
          })],
        }
      }
      None => Block {
        span: node.span,
        body: Vec::new(),
      },
    };

    Ok(IrExpr::FunctionExpr(FunctionDecl {
      span: node.span,
      name: None,
      params,
      body,
      annotations: Annotations {
        is_async,
        ..Default::default()
      },
      is_arrow: true,
    }))
  }

  pub fn lower_await_expression(
    &mut self,
    node: &crate::cst::CstNode,
  ) -> Result<IrExpr, LowerError> {
    let argument = self
      .first_named_child(node)
      .map(|a| self.lower_expression(a))
      .transpose()?
      .unwrap_or(IrExpr::Opaque(OpaqueExpr {
        span: node.span,
        cst_kind: "missing_await_argument".to_string(),
        text: String::new(),
      }));

    Ok(IrExpr::AwaitExpr(AwaitExpr {
      span: node.span,
      argument: Box::new(argument),
    }))
  }

  pub fn lower_yield_expression(
    &mut self,
    node: &crate::cst::CstNode,
  ) -> Result<IrExpr, LowerError> {
    let delegate = node.children.iter().any(|c| !c.named && c.kind == "*");

    let argument = self
      .first_named_child(node)
      .map(|a| self.lower_expression(a))
      .transpose()?
      .map(Box::new);

    Ok(IrExpr::YieldExpr(YieldExpr {
      span: node.span,
      argument,
      delegate,
    }))
  }

  pub fn lower_spread_expression(
    &mut self,
    node: &crate::cst::CstNode,
  ) -> Result<IrExpr, LowerError> {
    let argument = self
      .first_named_child(node)
      .map(|a| self.lower_expression(a))
      .transpose()?
      .unwrap_or(IrExpr::Opaque(OpaqueExpr {
        span: node.span,
        cst_kind: "missing_spread_argument".to_string(),
        text: String::new(),
      }));

    Ok(IrExpr::SpreadExpr(SpreadExpr {
      span: node.span,
      argument: Box::new(argument),
    }))
  }
}
