use ehrmantraut_core::ir::{
  AccessorKind, AccessorProperty, Annotations, ArrayExpr, Assignment, AwaitExpr, BinaryExpr, Block,
  Call, ConditionalExpr, FunctionDecl, Identifier, IrExpr, IrNode, KeyValueProperty, Literal,
  LiteralValue, MemberAccess, MethodProperty, NewExpr, ObjectExpr, ObjectProperty, OpaqueExpr,
  Param, ReturnStmt, ShorthandProperty, SpreadExpr, SpreadProperty, TaggedTemplate, TemplateLiteral,
  UnaryExpr, UpdateExpr, YieldExpr,
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

    // Tagged template: tree-sitter represents `tag`hello`` as a call_expression
    // where the arguments field is a template_string instead of an arguments node.
    if let Some(args) = args_node {
      if args.kind == "template_string" {
        let quasi = self.lower_template_string(args)?;
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

  // ── Unary / Update ──────────────────────────────────────────────

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

    // Determine prefix vs postfix by checking if operator comes before operand
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

  // ── Function expressions ──────────────────────────────────────

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

  // ── Arrow function ────────────────────────────────────────────

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

  // ── Template literal ──────────────────────────────────────────

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

    Ok(IrExpr::TemplateLiteral(TemplateLiteral {
      span: node.span,
      quasis,
      expressions,
    }))
  }

  // ── Await / Yield ─────────────────────────────────────────────

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

  // ── New expression ────────────────────────────────────────────

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

  // ── Spread ────────────────────────────────────────────────────

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

  // ── Conditional (ternary) ───────────────────────────────────────

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

  // ── Array literal ───────────────────────────────────────────────

  pub fn lower_array_expression(
    &mut self,
    node: &crate::cst::CstNode,
  ) -> Result<IrExpr, LowerError> {
    let mut elements: Vec<Option<IrExpr>> = Vec::new();
    let mut prev_was_comma = false;
    let mut first = true;

    for child in &node.children {
      match child.kind.as_str() {
        "[" => {
          first = true;
          continue;
        }
        "]" => {
          // Trailing comma before ] does NOT create a hole
          break;
        }
        "," => {
          if first || prev_was_comma {
            // Hole: either leading comma or consecutive commas
            elements.push(None);
          }
          prev_was_comma = true;
          first = false;
          continue;
        }
        _ => {
          if !child.named {
            continue;
          }
          let expr = self.lower_expression(child)?;
          elements.push(Some(expr));
          prev_was_comma = false;
          first = false;
        }
      }
    }

    Ok(IrExpr::ArrayExpr(ArrayExpr {
      span: node.span,
      elements,
    }))
  }

  // ── Object literal ──────────────────────────────────────────────

  pub fn lower_object_expression(
    &mut self,
    node: &crate::cst::CstNode,
  ) -> Result<IrExpr, LowerError> {
    let mut properties = Vec::new();

    for child in &node.children {
      if !child.named {
        continue;
      }
      match child.kind.as_str() {
        "pair" => {
          properties.push(self.lower_pair_property(child)?);
        }
        "shorthand_property_identifier" | "shorthand_property_identifier_pattern" => {
          properties.push(ObjectProperty::Shorthand(ShorthandProperty {
            span: child.span,
            name: self.node_text(child),
          }));
        }
        "method_definition" => {
          properties.push(self.lower_method_or_accessor_property(child)?);
        }
        "spread_element" => {
          let argument = self
            .first_named_child(child)
            .map(|a| self.lower_expression(a))
            .transpose()?
            .unwrap_or(IrExpr::Opaque(OpaqueExpr {
              span: child.span,
              cst_kind: "missing_spread_argument".to_string(),
              text: String::new(),
            }));
          properties.push(ObjectProperty::Spread(SpreadProperty {
            span: child.span,
            argument,
          }));
        }
        _ => {
          // Unknown property type — try as key-value fallback
          properties.push(ObjectProperty::Shorthand(ShorthandProperty {
            span: child.span,
            name: self.node_text(child),
          }));
        }
      }
    }

    Ok(IrExpr::ObjectExpr(ObjectExpr {
      span: node.span,
      properties,
    }))
  }

  fn lower_pair_property(
    &mut self,
    node: &crate::cst::CstNode,
  ) -> Result<ObjectProperty, LowerError> {
    let key_node = self.child_by_field(node, "key");
    let value_node = self.child_by_field(node, "value");

    let computed = key_node
      .map(|k| k.kind == "computed_property_name")
      .unwrap_or(false);

    let key = match key_node {
      Some(k) => {
        if k.kind == "computed_property_name" {
          match self.first_named_child(k) {
            Some(inner) => self.lower_expression(inner)?,
            None => self.lower_expression(k)?,
          }
        } else {
          self.lower_expression(k)?
        }
      }
      None => IrExpr::Opaque(OpaqueExpr {
        span: node.span,
        cst_kind: "missing_key".to_string(),
        text: String::new(),
      }),
    };

    let value = match value_node {
      Some(v) => self.lower_expression(v)?,
      None => IrExpr::Opaque(OpaqueExpr {
        span: node.span,
        cst_kind: "missing_value".to_string(),
        text: String::new(),
      }),
    };

    Ok(ObjectProperty::KeyValue(KeyValueProperty {
      span: node.span,
      key,
      value,
      computed,
    }))
  }

  fn lower_method_or_accessor_property(
    &mut self,
    node: &crate::cst::CstNode,
  ) -> Result<ObjectProperty, LowerError> {
    // Check for get/set accessor
    let has_get = node.children.iter().any(|c| !c.named && c.kind == "get");
    let has_set = node.children.iter().any(|c| !c.named && c.kind == "set");

    let name_node = self.child_by_field(node, "name");
    let computed = name_node
      .map(|n| n.kind == "computed_property_name")
      .unwrap_or(false);

    let key = match name_node {
      Some(n) => {
        if n.kind == "computed_property_name" {
          match self.first_named_child(n) {
            Some(inner) => self.lower_expression(inner)?,
            None => self.lower_expression(n)?,
          }
        } else {
          self.lower_expression(n)?
        }
      }
      None => IrExpr::Opaque(OpaqueExpr {
        span: node.span,
        cst_kind: "missing_name".to_string(),
        text: String::new(),
      }),
    };

    let params = self.lower_formal_parameters(node);

    let body_node = self.child_by_field(node, "body");
    let body = match body_node {
      Some(b) => self.lower_block(b)?,
      None => Block {
        span: node.span,
        body: Vec::new(),
      },
    };

    if has_get || has_set {
      let accessor_kind = if has_get {
        AccessorKind::Get
      } else {
        AccessorKind::Set
      };
      return Ok(ObjectProperty::Accessor(AccessorProperty {
        span: node.span,
        key,
        accessor_kind,
        params,
        body,
      }));
    }

    let is_async = node.children.iter().any(|c| !c.named && c.kind == "async");
    let is_generator = node.children.iter().any(|c| !c.named && c.kind == "*");

    Ok(ObjectProperty::Method(MethodProperty {
      span: node.span,
      key,
      params,
      body,
      computed,
      is_async,
      is_generator,
    }))
  }

  // ── Helpers ──────────────────────────────────────────────────────

  fn has_optional_chain(node: &crate::cst::CstNode) -> bool {
    node
      .children
      .iter()
      .any(|c| c.kind == "optional_chain" || c.field_name.as_deref() == Some("optional_chain"))
  }
}
