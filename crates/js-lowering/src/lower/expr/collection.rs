use ehrmantraut_core::ir::{
  AccessorKind, AccessorProperty, ArrayExpr, Block, IrExpr, KeyValueProperty, MethodProperty,
  ObjectExpr, ObjectProperty, OpaqueExpr, ShorthandProperty, SpreadProperty,
};

use crate::lower::{JsLowerer, LowerError};

impl JsLowerer {
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
          properties.push(ObjectProperty::Opaque(OpaqueExpr {
            span: child.span,
            cst_kind: child.kind.clone(),
            text: self.node_text(child),
          }));
        }
      }
    }

    Ok(IrExpr::ObjectExpr(ObjectExpr {
      span: node.span,
      properties,
    }))
  }

  pub(crate) fn lower_pair_property(
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

  pub(crate) fn lower_method_or_accessor_property(
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
}
