mod control_flow;
mod declarations;
mod expressions;

use std::fmt;

use ehrmantraut_core::ir::{
  Block, DeclKind, ExpressionStatement, IrExpr, IrModule, IrNode, OpaqueExpr, OpaqueNode,
  ScopeLevel,
};

use crate::cst::CstNode;
use crate::parser::Language;

#[derive(Debug)]
pub enum LowerError {
  UnexpectedNode { kind: String, expected: String },
}

impl fmt::Display for LowerError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      LowerError::UnexpectedNode { kind, expected } => {
        write!(f, "unexpected node '{kind}', expected {expected}")
      }
    }
  }
}

impl std::error::Error for LowerError {}

pub fn lower(cst: &CstNode, language: Language, source: &str) -> Result<IrModule, LowerError> {
  let mut lowerer = JsLowerer::new(language, source);
  lowerer.lower_program(cst)
}

pub(crate) struct JsLowerer {
  language: Language,
  source: String,
}

impl JsLowerer {
  pub fn new(language: Language, source: &str) -> Self {
    Self {
      language,
      source: source.to_string(),
    }
  }

  pub fn lower_program(&mut self, cst: &CstNode) -> Result<IrModule, LowerError> {
    let body = self.lower_children(cst)?;
    Ok(IrModule {
      span: cst.span,
      body,
    })
  }

  pub fn lower_children(&mut self, parent: &CstNode) -> Result<Vec<IrNode>, LowerError> {
    let mut nodes = Vec::new();
    for child in &parent.children {
      // Variable declarations can produce multiple nodes (multi-declarator)
      if child.kind == "lexical_declaration" || child.kind == "variable_declaration" {
        nodes.extend(self.lower_variable_declaration(child)?);
        continue;
      }
      // Export statements containing variable declarations need multi-declarator handling
      if child.kind == "export_statement" {
        if let Some(decl) = child
          .children
          .iter()
          .find(|c| c.kind == "lexical_declaration" || c.kind == "variable_declaration")
        {
          let mut var_nodes = self.lower_variable_declaration(decl)?;
          for var_node in &mut var_nodes {
            if let IrNode::VariableDecl(ref mut v) = var_node {
              v.annotations.is_export = true;
              v.annotations.declaration_kind = v
                .annotations
                .declaration_kind
                .clone()
                .or(Some(DeclKind::Import));
            }
          }
          nodes.extend(var_nodes);
          continue;
        }
      }
      if let Some(node) = self.lower_node(child)? {
        nodes.push(node);
      }
    }
    Ok(nodes)
  }

  pub fn lower_node(&mut self, node: &CstNode) -> Result<Option<IrNode>, LowerError> {
    // Skip unnamed nodes (punctuation, keywords, operators)
    if !node.named {
      return Ok(None);
    }

    // Skip TypeScript-only type nodes
    if self.language != Language::JavaScript && Self::is_ts_type_node(&node.kind) {
      return Ok(None);
    }

    match node.kind.as_str() {
      // Declarations (multi-declarator handled in lower_children)
      "lexical_declaration" | "variable_declaration" => {
        Ok(self.lower_variable_declaration(node)?.into_iter().next())
      }
      "function_declaration" | "generator_function_declaration" => {
        Ok(Some(self.lower_function_declaration(node)?))
      }
      "class_declaration" => Ok(Some(self.lower_class_declaration(node)?)),
      "import_statement" => Ok(Some(self.lower_import_declaration(node)?)),
      "export_statement" => Ok(Some(self.lower_export_declaration(node)?)),

      // Control flow
      "if_statement" => Ok(Some(self.lower_if_statement(node)?)),
      "for_statement" => Ok(Some(self.lower_for_statement(node)?)),
      "for_in_statement" => Ok(Some(self.lower_for_in_statement(node)?)),
      "while_statement" => Ok(Some(self.lower_while_statement(node)?)),
      "do_statement" => Ok(Some(self.lower_do_while_statement(node)?)),
      "switch_statement" => Ok(Some(self.lower_switch_statement(node)?)),
      "try_statement" => Ok(Some(self.lower_try_statement(node)?)),
      "return_statement" => Ok(Some(self.lower_return_statement(node)?)),
      "throw_statement" => Ok(Some(self.lower_throw_statement(node)?)),
      "break_statement" => Ok(Some(self.lower_break_statement(node)?)),
      "continue_statement" => Ok(Some(self.lower_continue_statement(node)?)),

      // Expressions as statements
      "expression_statement" => {
        let expr_child = self.first_named_child(node);
        match expr_child {
          Some(child) => {
            let expr = self.lower_expression(child)?;
            Ok(Some(IrNode::ExpressionStatement(ExpressionStatement {
              span: node.span,
              expression: expr,
            })))
          }
          None => Ok(None),
        }
      }

      // Block
      "statement_block" => {
        let block = self.lower_block(node)?;
        Ok(Some(IrNode::Block(block)))
      }

      // Empty statement
      "empty_statement" => Ok(None),

      // Comment nodes
      "comment" => Ok(None),

      // Fallback: opaque node
      _ => Ok(Some(IrNode::Opaque(OpaqueNode {
        span: node.span,
        cst_kind: node.kind.clone(),
        text: self.node_text(node),
      }))),
    }
  }

  pub fn lower_expression(&mut self, node: &CstNode) -> Result<IrExpr, LowerError> {
    // Skip TypeScript type nodes within expressions
    if self.language != Language::JavaScript && Self::is_ts_type_node(&node.kind) {
      return Ok(IrExpr::Opaque(OpaqueExpr {
        span: node.span,
        cst_kind: node.kind.clone(),
        text: self.node_text(node),
      }));
    }

    match node.kind.as_str() {
      "assignment_expression" | "augmented_assignment_expression" => {
        self.lower_assignment_expression(node)
      }
      "call_expression" => self.lower_call_expression(node),
      "member_expression" => self.lower_member_expression(node),
      "subscript_expression" => self.lower_subscript_expression(node),
      "binary_expression" => self.lower_binary_expression(node),
      "identifier" | "shorthand_property_identifier" => Ok(self.lower_identifier(node)),
      "property_identifier" => Ok(self.lower_identifier(node)),
      "this" | "super" => Ok(self.lower_identifier(node)),
      "number" => Ok(self.lower_number_literal(node)),
      "string" => Ok(self.lower_string_literal(node)),
      "template_string" => {
        let text = self.node_text(node);
        if text.contains("${") {
          Ok(IrExpr::Opaque(OpaqueExpr {
            span: node.span,
            cst_kind: node.kind.clone(),
            text,
          }))
        } else {
          Ok(self.lower_string_literal(node))
        }
      }
      "true" | "false" => Ok(self.lower_boolean_literal(node)),
      "null" => Ok(self.lower_null_literal(node)),
      "undefined" => Ok(self.lower_undefined(node)),
      "unary_expression" => self.lower_unary_expression(node),
      "update_expression" => self.lower_update_expression(node),
      "ternary_expression" => self.lower_conditional_expression(node),
      "arrow_function" => self.lower_arrow_function(node),
      "function_expression" => self.lower_function_expression(node),
      "generator_function" => self.lower_function_expression(node),
      "new_expression" => self.lower_new_expression(node),
      "await_expression" => self.lower_await_expression(node),
      "yield_expression" => self.lower_yield_expression(node),
      "spread_element" => self.lower_spread_expression(node),
      "array" => self.lower_array_expression(node),
      "object" => self.lower_object_expression(node),
      "parenthesized_expression" => {
        // Unwrap parenthesized expression
        match self.first_named_child(node) {
          Some(child) => self.lower_expression(child),
          None => Ok(IrExpr::Opaque(OpaqueExpr {
            span: node.span,
            cst_kind: node.kind.clone(),
            text: self.node_text(node),
          })),
        }
      }
      // Fallback
      _ => Ok(IrExpr::Opaque(OpaqueExpr {
        span: node.span,
        cst_kind: node.kind.clone(),
        text: self.node_text(node),
      })),
    }
  }

  pub fn lower_block(&mut self, node: &CstNode) -> Result<Block, LowerError> {
    let body = self.lower_children(node)?;
    Ok(Block {
      span: node.span,
      body,
    })
  }

  // ── Helpers ──────────────────────────────────────────────────────

  pub(crate) fn first_named_child<'a>(&self, node: &'a CstNode) -> Option<&'a CstNode> {
    node.children.iter().find(|c| c.named)
  }

  pub(crate) fn child_by_field<'a>(&self, node: &'a CstNode, field: &str) -> Option<&'a CstNode> {
    node
      .children
      .iter()
      .find(|c| c.field_name.as_deref() == Some(field))
  }

  pub(crate) fn node_text(&self, node: &CstNode) -> String {
    if let Some(text) = &node.text {
      return text.clone();
    }
    // For branch nodes, reconstruct from source if available
    if !self.source.is_empty() {
      let start = node.span.start.offset as usize;
      let end = node.span.end.offset as usize;
      if end <= self.source.len() {
        return self.source[start..end].to_string();
      }
    }
    // Fallback: concatenate leaf text
    self.collect_text(node)
  }

  fn collect_text(&self, node: &CstNode) -> String {
    if let Some(text) = &node.text {
      return text.clone();
    }
    node
      .children
      .iter()
      .map(|c| self.collect_text(c))
      .collect::<Vec<_>>()
      .join("")
  }

  pub(crate) fn decl_kind_and_scope(kind_str: &str) -> (Option<DeclKind>, Option<ScopeLevel>) {
    match kind_str {
      "let" => (Some(DeclKind::Let), Some(ScopeLevel::Block)),
      "const" => (Some(DeclKind::Const), Some(ScopeLevel::Block)),
      "var" => (Some(DeclKind::Var), Some(ScopeLevel::Function)),
      _ => (None, None),
    }
  }

  fn is_ts_type_node(kind: &str) -> bool {
    matches!(
      kind,
      "type_annotation"
        | "type_parameters"
        | "type_arguments"
        | "interface_declaration"
        | "type_alias_declaration"
        | "enum_declaration"
        | "as_expression"
        | "satisfies_expression"
        | "non_null_expression"
        | "type_assertion"
    )
  }
}
