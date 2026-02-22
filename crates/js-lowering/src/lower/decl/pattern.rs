use ehrmantraut_core::ir::{
  ArrayPattern, AssignmentPattern, ObjectPattern, ObjectPatternProperty, Pattern, PatternKeyValue,
  PatternRest, PatternShorthand, RestPattern,
};

use crate::lower::{JsLowerer, LowerError};

impl JsLowerer {
  pub(crate) fn lower_pattern(
    &mut self,
    node: &crate::cst::CstNode,
  ) -> Result<Pattern, LowerError> {
    match node.kind.as_str() {
      "object_pattern" => self.lower_object_pattern(node),
      "array_pattern" => self.lower_array_pattern(node),
      "assignment_pattern" => {
        let left = self.child_by_field(node, "left");
        let right = self.child_by_field(node, "right");

        let left_pat = match left {
          Some(l) => self.lower_pattern(l)?,
          None => Pattern::Object(ObjectPattern {
            span: node.span,
            properties: Vec::new(),
          }),
        };

        let right_expr = match right {
          Some(r) => self.lower_expression(r)?,
          None => ehrmantraut_core::ir::IrExpr::Opaque(ehrmantraut_core::ir::OpaqueExpr {
            span: node.span,
            cst_kind: "missing_default".to_string(),
            text: String::new(),
          }),
        };

        Ok(Pattern::Assignment(AssignmentPattern {
          span: node.span,
          left: Box::new(left_pat),
          right: right_expr,
        }))
      }
      "rest_pattern" => {
        let inner = self
          .first_named_child(node)
          .map(|n| self.lower_pattern(n))
          .transpose()?
          .unwrap_or(Pattern::Object(ObjectPattern {
            span: node.span,
            properties: Vec::new(),
          }));

        Ok(Pattern::Rest(RestPattern {
          span: node.span,
          argument: Box::new(inner),
        }))
      }
      // Simple identifier treated as a shorthand pattern
      "identifier" => Ok(Pattern::Object(ObjectPattern {
        span: node.span,
        properties: vec![ObjectPatternProperty::Shorthand(PatternShorthand {
          span: node.span,
          name: self.node_text(node),
          default_value: None,
        })],
      })),
      _ => Ok(Pattern::Opaque(ehrmantraut_core::ir::OpaqueExpr {
        span: node.span,
        cst_kind: node.kind.clone(),
        text: self.node_text(node),
      })),
    }
  }

  fn lower_object_pattern(&mut self, node: &crate::cst::CstNode) -> Result<Pattern, LowerError> {
    let mut properties = Vec::new();

    for child in &node.children {
      if !child.named {
        continue;
      }
      match child.kind.as_str() {
        "shorthand_property_identifier_pattern" => {
          properties.push(ObjectPatternProperty::Shorthand(PatternShorthand {
            span: child.span,
            name: self.node_text(child),
            default_value: None,
          }));
        }
        "pair_pattern" => {
          let key = self.child_by_field(child, "key");
          let value = self.child_by_field(child, "value");

          let key_name = key.map(|k| self.node_text(k)).unwrap_or_default();
          let value_pat = match value {
            Some(v) => self.lower_pattern(v)?,
            None => Pattern::Object(ObjectPattern {
              span: child.span,
              properties: Vec::new(),
            }),
          };

          properties.push(ObjectPatternProperty::KeyValue(PatternKeyValue {
            span: child.span,
            key: key_name,
            value: value_pat,
          }));
        }
        "rest_pattern" => {
          let inner = self
            .first_named_child(child)
            .map(|n| self.node_text(n))
            .unwrap_or_default();
          properties.push(ObjectPatternProperty::Rest(PatternRest {
            span: child.span,
            name: inner,
          }));
        }
        "assignment_pattern" | "object_assignment_pattern" => {
          // shorthand with default: { x = 10 }
          let left = self.child_by_field(child, "left");
          let right = self.child_by_field(child, "right");

          let name = left.map(|l| self.node_text(l)).unwrap_or_default();
          let default_value = right.map(|r| self.lower_expression(r)).transpose()?;

          properties.push(ObjectPatternProperty::Shorthand(PatternShorthand {
            span: child.span,
            name,
            default_value,
          }));
        }
        _ => {}
      }
    }

    Ok(Pattern::Object(ObjectPattern {
      span: node.span,
      properties,
    }))
  }

  fn lower_array_pattern(&mut self, node: &crate::cst::CstNode) -> Result<Pattern, LowerError> {
    let mut elements: Vec<Option<Pattern>> = Vec::new();
    let mut prev_was_comma = false;
    let mut first = true;

    for child in &node.children {
      match child.kind.as_str() {
        "[" => {
          first = true;
          continue;
        }
        "]" => break,
        "," => {
          if first || prev_was_comma {
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
          let pat = self.lower_pattern(child)?;
          elements.push(Some(pat));
          prev_was_comma = false;
          first = false;
        }
      }
    }

    Ok(Pattern::Array(ArrayPattern {
      span: node.span,
      elements,
    }))
  }
}
