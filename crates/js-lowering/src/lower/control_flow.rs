use ehrmantraut_core::ir::{
  Block, BreakStmt, CatchClause, ContinueStmt, ForAnnotations, ForKind, ForStmt, IfStmt, IrNode,
  ReturnStmt, SwitchCase, SwitchStmt, TryCatchStmt, WhileStmt,
};

use super::{JsLowerer, LowerError};

impl JsLowerer {
  pub fn lower_if_statement(&mut self, node: &crate::cst::CstNode) -> Result<IrNode, LowerError> {
    let condition_node = self.child_by_field(node, "condition");
    let condition = match condition_node {
      Some(c) => {
        // condition is wrapped in parenthesized_expression
        match self.first_named_child(c) {
          Some(inner) => self.lower_expression(inner)?,
          None => self.lower_expression(c)?,
        }
      }
      None => {
        return Ok(IrNode::Opaque(ehrmantraut_core::ir::OpaqueNode {
          span: node.span,
          cst_kind: node.kind.clone(),
          text: self.node_text(node),
        }))
      }
    };

    let consequent_node = self.child_by_field(node, "consequence");
    let consequent = match consequent_node {
      Some(c) => self.lower_block_or_wrap(c)?,
      None => Block {
        span: node.span,
        body: Vec::new(),
      },
    };

    let alternate = self
      .child_by_field(node, "alternative")
      .map(|alt| {
        if alt.kind == "else_clause" {
          // else clause wraps a statement_block or another if_statement
          match self.first_named_child(alt) {
            Some(inner) => self.lower_node(inner),
            None => Ok(None),
          }
        } else {
          self.lower_node(alt)
        }
      })
      .transpose()?
      .flatten()
      .map(Box::new);

    Ok(IrNode::If(IfStmt {
      span: node.span,
      condition,
      consequent,
      alternate,
    }))
  }

  pub fn lower_for_statement(&mut self, node: &crate::cst::CstNode) -> Result<IrNode, LowerError> {
    let init = self
      .child_by_field(node, "initializer")
      .map(|n| self.lower_node(n))
      .transpose()?
      .flatten()
      .map(Box::new);

    let condition = self
      .child_by_field(node, "condition")
      .map(|n| self.lower_expression(n))
      .transpose()?;

    let update = self
      .child_by_field(node, "increment")
      .map(|n| self.lower_expression(n))
      .transpose()?;

    let body_node = self.child_by_field(node, "body");
    let body = match body_node {
      Some(b) => self.lower_block_or_wrap(b)?,
      None => Block {
        span: node.span,
        body: Vec::new(),
      },
    };

    Ok(IrNode::For(ForStmt {
      span: node.span,
      init,
      condition,
      update,
      body,
      annotations: ForAnnotations {
        kind: ForKind::Standard,
      },
    }))
  }

  pub fn lower_for_in_statement(
    &mut self,
    node: &crate::cst::CstNode,
  ) -> Result<IrNode, LowerError> {
    // Determine if this is for..in or for..of
    let is_of = node.kind == "for_in_statement" && node.children.iter().any(|c| c.kind == "of");
    let kind = if is_of { ForKind::Of } else { ForKind::In };

    let left = self
      .child_by_field(node, "left")
      .map(|n| self.lower_node(n))
      .transpose()?
      .flatten()
      .map(Box::new);

    let right = self
      .child_by_field(node, "right")
      .map(|n| self.lower_expression(n))
      .transpose()?;

    let body_node = self.child_by_field(node, "body");
    let body = match body_node {
      Some(b) => self.lower_block_or_wrap(b)?,
      None => Block {
        span: node.span,
        body: Vec::new(),
      },
    };

    Ok(IrNode::For(ForStmt {
      span: node.span,
      init: left,
      condition: right,
      update: None,
      body,
      annotations: ForAnnotations { kind },
    }))
  }

  pub fn lower_while_statement(
    &mut self,
    node: &crate::cst::CstNode,
  ) -> Result<IrNode, LowerError> {
    let condition_node = self.child_by_field(node, "condition");
    let condition = match condition_node {
      Some(c) => match self.first_named_child(c) {
        Some(inner) => self.lower_expression(inner)?,
        None => self.lower_expression(c)?,
      },
      None => {
        return Ok(IrNode::Opaque(ehrmantraut_core::ir::OpaqueNode {
          span: node.span,
          cst_kind: node.kind.clone(),
          text: self.node_text(node),
        }))
      }
    };

    let body_node = self.child_by_field(node, "body");
    let body = match body_node {
      Some(b) => self.lower_block_or_wrap(b)?,
      None => Block {
        span: node.span,
        body: Vec::new(),
      },
    };

    Ok(IrNode::While(WhileStmt {
      span: node.span,
      condition,
      body,
      is_do_while: false,
    }))
  }

  pub fn lower_do_while_statement(
    &mut self,
    node: &crate::cst::CstNode,
  ) -> Result<IrNode, LowerError> {
    let condition_node = self.child_by_field(node, "condition");
    let condition = match condition_node {
      Some(c) => match self.first_named_child(c) {
        Some(inner) => self.lower_expression(inner)?,
        None => self.lower_expression(c)?,
      },
      None => {
        return Ok(IrNode::Opaque(ehrmantraut_core::ir::OpaqueNode {
          span: node.span,
          cst_kind: node.kind.clone(),
          text: self.node_text(node),
        }))
      }
    };

    let body_node = self.child_by_field(node, "body");
    let body = match body_node {
      Some(b) => self.lower_block_or_wrap(b)?,
      None => Block {
        span: node.span,
        body: Vec::new(),
      },
    };

    Ok(IrNode::While(WhileStmt {
      span: node.span,
      condition,
      body,
      is_do_while: true,
    }))
  }

  pub fn lower_switch_statement(
    &mut self,
    node: &crate::cst::CstNode,
  ) -> Result<IrNode, LowerError> {
    let discriminant_node = self.child_by_field(node, "value");
    let discriminant = match discriminant_node {
      Some(d) => match self.first_named_child(d) {
        Some(inner) => self.lower_expression(inner)?,
        None => self.lower_expression(d)?,
      },
      None => {
        return Ok(IrNode::Opaque(ehrmantraut_core::ir::OpaqueNode {
          span: node.span,
          cst_kind: node.kind.clone(),
          text: self.node_text(node),
        }))
      }
    };

    let body_node = self.child_by_field(node, "body");
    let cases = match body_node {
      Some(b) => b
        .children
        .iter()
        .filter(|c| c.kind == "switch_case" || c.kind == "switch_default")
        .map(|c| self.lower_switch_case(c))
        .collect::<Result<Vec<_>, _>>()?,
      None => Vec::new(),
    };

    Ok(IrNode::Switch(SwitchStmt {
      span: node.span,
      discriminant,
      cases,
    }))
  }

  fn lower_switch_case(&mut self, node: &crate::cst::CstNode) -> Result<SwitchCase, LowerError> {
    let test = if node.kind == "switch_default" {
      None
    } else {
      self
        .child_by_field(node, "value")
        .map(|v| self.lower_expression(v))
        .transpose()?
    };

    let body = node
      .children
      .iter()
      .filter(|c| c.named && c.field_name.as_deref() != Some("value"))
      .filter_map(|c| self.lower_node(c).transpose())
      .collect::<Result<Vec<_>, _>>()?;

    Ok(SwitchCase {
      span: node.span,
      test,
      body,
    })
  }

  pub fn lower_try_statement(&mut self, node: &crate::cst::CstNode) -> Result<IrNode, LowerError> {
    let try_block = self
      .child_by_field(node, "body")
      .map(|b| self.lower_block(b))
      .transpose()?
      .unwrap_or(Block {
        span: node.span,
        body: Vec::new(),
      });

    let catch_clause = self
      .child_by_field(node, "handler")
      .map(|h| self.lower_catch_clause(h))
      .transpose()?;

    let finally_block = self
      .child_by_field(node, "finalizer")
      .map(|f| self.lower_block(f))
      .transpose()?;

    Ok(IrNode::TryCatch(TryCatchStmt {
      span: node.span,
      try_block,
      catch_clause,
      finally_block,
    }))
  }

  fn lower_catch_clause(&mut self, node: &crate::cst::CstNode) -> Result<CatchClause, LowerError> {
    let param = self
      .child_by_field(node, "parameter")
      .and_then(|p| self.first_named_child(p).or(Some(p)))
      .map(|p| self.node_text(p));

    let body = self
      .child_by_field(node, "body")
      .map(|b| self.lower_block(b))
      .transpose()?
      .unwrap_or(Block {
        span: node.span,
        body: Vec::new(),
      });

    Ok(CatchClause {
      span: node.span,
      param,
      body,
    })
  }

  pub fn lower_return_statement(
    &mut self,
    node: &crate::cst::CstNode,
  ) -> Result<IrNode, LowerError> {
    let value = self
      .first_named_child(node)
      .map(|v| self.lower_expression(v))
      .transpose()?;

    Ok(IrNode::Return(ReturnStmt {
      span: node.span,
      value,
    }))
  }

  pub fn lower_break_statement(
    &mut self,
    node: &crate::cst::CstNode,
  ) -> Result<IrNode, LowerError> {
    let label = self
      .child_by_field(node, "label")
      .map(|l| self.node_text(l));

    Ok(IrNode::Break(BreakStmt {
      span: node.span,
      label,
    }))
  }

  pub fn lower_continue_statement(
    &mut self,
    node: &crate::cst::CstNode,
  ) -> Result<IrNode, LowerError> {
    let label = self
      .child_by_field(node, "label")
      .map(|l| self.node_text(l));

    Ok(IrNode::Continue(ContinueStmt {
      span: node.span,
      label,
    }))
  }

  // Helper: lower a node as a block, wrapping single statements if needed
  pub(crate) fn lower_block_or_wrap(
    &mut self,
    node: &crate::cst::CstNode,
  ) -> Result<Block, LowerError> {
    if node.kind == "statement_block" {
      self.lower_block(node)
    } else {
      // Single statement body (e.g., `for (...) x++`)
      let inner = self.lower_node(node)?;
      Ok(Block {
        span: node.span,
        body: inner.into_iter().collect(),
      })
    }
  }
}
