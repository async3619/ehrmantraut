use ehrmantraut_core::ir::{
  Accessibility, Annotations, ArrayPattern, AssignmentPattern, Block, ClassDecl, DeclKind,
  ExportDecl, ExportSpecifier, FunctionDecl, ImportDecl, ImportDefault, ImportNamed,
  ImportNamespace, ImportSpecifier, IrNode, MethodDefinition, MethodKind, ObjectPattern,
  ObjectPatternProperty, Param, Pattern, PatternKeyValue, PatternRest, PatternShorthand,
  PropertyDefinition, RestPattern, ScopeLevel, VariableDecl,
};

use super::{JsLowerer, LowerError};

impl JsLowerer {
  pub fn lower_variable_declaration(
    &mut self,
    node: &crate::cst::CstNode,
  ) -> Result<Vec<IrNode>, LowerError> {
    let kind_text = node
      .children
      .iter()
      .find(|c| !c.named && matches!(c.kind.as_str(), "let" | "const" | "var"))
      .map(|c| c.kind.as_str())
      .unwrap_or("");

    let (decl_kind, scope_level) = Self::decl_kind_and_scope(kind_text);

    let declarators: Vec<_> = node
      .children
      .iter()
      .filter(|c| c.kind == "variable_declarator")
      .collect();

    let mut nodes = Vec::new();
    for declarator in &declarators {
      let name_node = self.child_by_field(declarator, "name");

      let (name, pattern) = match name_node {
        Some(n) if n.kind == "object_pattern" || n.kind == "array_pattern" => {
          let pat = self.lower_pattern(n)?;
          (String::new(), Some(pat))
        }
        Some(n) => (self.node_text(n), None),
        None => (String::new(), None),
      };

      let value = self
        .child_by_field(declarator, "value")
        .map(|v| self.lower_expression(v))
        .transpose()?;

      nodes.push(IrNode::VariableDecl(VariableDecl {
        span: declarator.span,
        name,
        value,
        pattern,
        annotations: Annotations {
          scope_level: scope_level.clone(),
          declaration_kind: decl_kind.clone(),
          ..Default::default()
        },
      }));
    }

    if nodes.is_empty() {
      nodes.push(IrNode::VariableDecl(VariableDecl {
        span: node.span,
        name: String::new(),
        value: None,
        pattern: None,
        annotations: Annotations {
          scope_level,
          declaration_kind: decl_kind,
          ..Default::default()
        },
      }));
    }

    Ok(nodes)
  }

  pub fn lower_function_declaration(
    &mut self,
    node: &crate::cst::CstNode,
  ) -> Result<IrNode, LowerError> {
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

    Ok(IrNode::FunctionDecl(FunctionDecl {
      span: node.span,
      name,
      params,
      body,
      annotations: Annotations {
        scope_level: Some(ScopeLevel::Module),
        declaration_kind: Some(DeclKind::Function),
        is_async,
        is_generator,
        ..Default::default()
      },
      is_arrow: false,
    }))
  }

  pub fn lower_class_declaration(
    &mut self,
    node: &crate::cst::CstNode,
  ) -> Result<IrNode, LowerError> {
    let name = self.child_by_field(node, "name").map(|n| self.node_text(n));

    let super_class = node
      .children
      .iter()
      .find(|c| c.field_name.as_deref() == Some("superclass"))
      .or_else(|| {
        node
          .children
          .iter()
          .find(|c| c.kind == "class_heritage")
          .and_then(|h| self.first_named_child(h))
      });

    let super_class_expr = super_class.map(|s| self.lower_expression(s)).transpose()?;

    let body_node = self.child_by_field(node, "body");
    let body = match body_node {
      Some(b) => self.lower_children(b)?,
      None => Vec::new(),
    };

    Ok(IrNode::ClassDecl(ClassDecl {
      span: node.span,
      name,
      super_class: super_class_expr,
      body,
      annotations: Annotations {
        declaration_kind: Some(DeclKind::Class),
        ..Default::default()
      },
    }))
  }

  // ── Import / Export ──────────────────────────────────────────────

  pub fn lower_import_declaration(
    &mut self,
    node: &crate::cst::CstNode,
  ) -> Result<IrNode, LowerError> {
    let source = self
      .child_by_field(node, "source")
      .map(|s| Self::strip_quotes(self.node_text(s)))
      .unwrap_or_default();

    let mut specifiers = Vec::new();

    for child in &node.children {
      match child.kind.as_str() {
        // import x from 'mod'
        "identifier" => {
          specifiers.push(ImportSpecifier::Default(ImportDefault {
            span: child.span,
            local: self.node_text(child),
          }));
        }
        // import { x, y as z } from 'mod'
        "import_clause" => {
          for inner in &child.children {
            match inner.kind.as_str() {
              "identifier" => {
                specifiers.push(ImportSpecifier::Default(ImportDefault {
                  span: inner.span,
                  local: self.node_text(inner),
                }));
              }
              "named_imports" => {
                self.collect_named_imports(inner, &mut specifiers);
              }
              "namespace_import" => {
                let local = self
                  .first_named_child(inner)
                  .map(|n| self.node_text(n))
                  .unwrap_or_default();
                specifiers.push(ImportSpecifier::Namespace(ImportNamespace {
                  span: inner.span,
                  local,
                }));
              }
              _ => {}
            }
          }
        }
        // import { x } from 'mod' (direct named_imports without import_clause)
        "named_imports" => {
          self.collect_named_imports(child, &mut specifiers);
        }
        // import * as ns from 'mod'
        "namespace_import" => {
          let local = self
            .first_named_child(child)
            .map(|n| self.node_text(n))
            .unwrap_or_default();
          specifiers.push(ImportSpecifier::Namespace(ImportNamespace {
            span: child.span,
            local,
          }));
        }
        _ => {}
      }
    }

    Ok(IrNode::ImportDecl(ImportDecl {
      span: node.span,
      specifiers,
      source,
    }))
  }

  fn collect_named_imports(
    &self,
    node: &crate::cst::CstNode,
    specifiers: &mut Vec<ImportSpecifier>,
  ) {
    for child in &node.children {
      if child.kind == "import_specifier" {
        let name_node = self.child_by_field(child, "name");
        let alias_node = self.child_by_field(child, "alias");

        let imported = name_node.map(|n| self.node_text(n)).unwrap_or_default();
        let local = alias_node
          .map(|a| self.node_text(a))
          .unwrap_or_else(|| imported.clone());

        specifiers.push(ImportSpecifier::Named(ImportNamed {
          span: child.span,
          imported,
          local,
        }));
      }
    }
  }

  pub fn lower_export_declaration(
    &mut self,
    node: &crate::cst::CstNode,
  ) -> Result<IrNode, LowerError> {
    let is_default = node
      .children
      .iter()
      .any(|c| !c.named && c.kind == "default");

    let source = self
      .child_by_field(node, "source")
      .map(|s| Self::strip_quotes(self.node_text(s)));

    // Check for declaration child (function, class)
    let decl_child = node.children.iter().find(|c| {
      matches!(
        c.kind.as_str(),
        "function_declaration" | "class_declaration"
      )
    });

    if let Some(decl) = decl_child {
      let mut inner = self.lower_node(decl)?;
      // Set export annotations on the declaration
      match &mut inner {
        Some(IrNode::FunctionDecl(ref mut f)) => {
          f.annotations.is_export = true;
          f.annotations.is_default = is_default;
        }
        Some(IrNode::ClassDecl(ref mut c)) => {
          c.annotations.is_export = true;
          c.annotations.is_default = is_default;
        }
        _ => {}
      }
      return Ok(
        inner.unwrap_or(IrNode::Opaque(ehrmantraut_core::ir::OpaqueNode {
          span: node.span,
          cst_kind: node.kind.clone(),
          text: self.node_text(node),
        })),
      );
    }

    // export { x, y as z } or export { x } from 'mod'
    let mut specifiers = Vec::new();
    if let Some(export_clause) = node.children.iter().find(|c| c.kind == "export_clause") {
      for child in &export_clause.children {
        if child.kind == "export_specifier" {
          let name_node = self.child_by_field(child, "name");
          let alias_node = self.child_by_field(child, "alias");

          let local = name_node.map(|n| self.node_text(n)).unwrap_or_default();
          let exported = alias_node
            .map(|a| self.node_text(a))
            .unwrap_or_else(|| local.clone());

          specifiers.push(ExportSpecifier {
            span: child.span,
            local,
            exported,
          });
        }
      }
    }

    // export default <expression>
    let declaration = if is_default {
      node
        .children
        .iter()
        .find(|c| c.named && c.kind != "export_clause" && c.kind != "string")
        .filter(|c| c.kind != "comment")
        .map(|c| {
          let expr = self.lower_expression(c)?;
          Ok(Box::new(IrNode::ExpressionStatement(
            ehrmantraut_core::ir::ExpressionStatement {
              span: c.span,
              expression: expr,
            },
          )))
        })
        .transpose()?
    } else {
      None
    };

    // export * from 'mod'
    let is_namespace_reexport = node.children.iter().any(|c| !c.named && c.kind == "*");
    if is_namespace_reexport {
      specifiers.push(ExportSpecifier {
        span: node.span,
        local: "*".to_string(),
        exported: "*".to_string(),
      });
    }

    Ok(IrNode::ExportDecl(ExportDecl {
      span: node.span,
      declaration,
      specifiers,
      source,
      is_default,
    }))
  }

  // ── Class members ─────────────────────────────────────────────

  pub fn lower_method_definition(
    &mut self,
    node: &crate::cst::CstNode,
  ) -> Result<IrNode, LowerError> {
    let is_static = node.children.iter().any(|c| !c.named && c.kind == "static");
    let has_get = node.children.iter().any(|c| !c.named && c.kind == "get");
    let has_set = node.children.iter().any(|c| !c.named && c.kind == "set");
    let is_async = node.children.iter().any(|c| !c.named && c.kind == "async");
    let is_generator = node.children.iter().any(|c| !c.named && c.kind == "*");

    let name_node = self.child_by_field(node, "name");

    let kind = if has_get {
      MethodKind::Get
    } else if has_set {
      MethodKind::Set
    } else if name_node.map(|n| self.node_text(n)).as_deref() == Some("constructor") {
      MethodKind::Constructor
    } else {
      MethodKind::Method
    };

    let computed = name_node
      .map(|n| n.kind == "computed_property_name")
      .unwrap_or(false);

    let key = match name_node {
      Some(n) if n.kind == "computed_property_name" => match self.first_named_child(n) {
        Some(inner) => self.lower_expression(inner)?,
        None => self.lower_expression(n)?,
      },
      Some(n) => self.lower_expression(n)?,
      None => ehrmantraut_core::ir::IrExpr::Opaque(ehrmantraut_core::ir::OpaqueExpr {
        span: node.span,
        cst_kind: "missing_method_name".to_string(),
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

    let accessibility = Self::extract_accessibility(node);

    Ok(IrNode::MethodDefinition(MethodDefinition {
      span: node.span,
      key,
      kind,
      params,
      body,
      is_static,
      computed,
      accessibility,
      is_async,
      is_generator,
    }))
  }

  pub fn lower_property_definition(
    &mut self,
    node: &crate::cst::CstNode,
  ) -> Result<IrNode, LowerError> {
    let is_static = node.children.iter().any(|c| !c.named && c.kind == "static");

    let name_node = self.child_by_field(node, "property");

    let computed = name_node
      .map(|n| n.kind == "computed_property_name")
      .unwrap_or(false);

    let key = match name_node {
      Some(n) if n.kind == "computed_property_name" => match self.first_named_child(n) {
        Some(inner) => self.lower_expression(inner)?,
        None => self.lower_expression(n)?,
      },
      Some(n) => self.lower_expression(n)?,
      None => ehrmantraut_core::ir::IrExpr::Opaque(ehrmantraut_core::ir::OpaqueExpr {
        span: node.span,
        cst_kind: "missing_property_name".to_string(),
        text: String::new(),
      }),
    };

    let value = self
      .child_by_field(node, "value")
      .map(|v| self.lower_expression(v))
      .transpose()?;

    let accessibility = Self::extract_accessibility(node);

    Ok(IrNode::PropertyDefinition(PropertyDefinition {
      span: node.span,
      key,
      value,
      is_static,
      computed,
      accessibility,
    }))
  }

  fn extract_accessibility(node: &crate::cst::CstNode) -> Option<Accessibility> {
    for child in &node.children {
      if !child.named {
        match child.kind.as_str() {
          "public" => return Some(Accessibility::Public),
          "private" => return Some(Accessibility::Private),
          "protected" => return Some(Accessibility::Protected),
          _ => {}
        }
      }
      // TypeScript accessibility modifiers may appear as named nodes
      if child.kind == "accessibility_modifier" {
        let text = if let Some(t) = &child.text {
          t.as_str()
        } else {
          ""
        };
        return match text {
          "public" => Some(Accessibility::Public),
          "private" => Some(Accessibility::Private),
          "protected" => Some(Accessibility::Protected),
          _ => None,
        };
      }
    }
    None
  }

  pub(crate) fn lower_formal_parameters(&mut self, func_node: &crate::cst::CstNode) -> Vec<Param> {
    let params_node = self.child_by_field(func_node, "parameters");
    match params_node {
      Some(p) => p
        .children
        .iter()
        .filter(|c| c.named)
        .filter_map(|c| self.lower_param(c).ok())
        .collect(),
      None => Vec::new(),
    }
  }

  pub(crate) fn lower_param(&mut self, node: &crate::cst::CstNode) -> Result<Param, LowerError> {
    match node.kind.as_str() {
      "object_pattern" | "array_pattern" => {
        let pat = self.lower_pattern(node)?;
        Ok(Param {
          span: node.span,
          name: String::new(),
          pattern: Some(pat),
          default_value: None,
        })
      }
      "assignment_pattern" => {
        let left = self.child_by_field(node, "left");
        let right = self.child_by_field(node, "right");

        let (name, pattern) = match left {
          Some(l) if l.kind == "object_pattern" || l.kind == "array_pattern" => {
            let pat = self.lower_pattern(l)?;
            (String::new(), Some(pat))
          }
          Some(l) => (self.node_text(l), None),
          None => (String::new(), None),
        };

        let default_value = right.map(|r| self.lower_expression(r)).transpose()?;

        Ok(Param {
          span: node.span,
          name,
          pattern,
          default_value,
        })
      }
      "rest_pattern" => {
        let inner = self.first_named_child(node);
        let name = inner.map(|n| self.node_text(n)).unwrap_or_default();
        Ok(Param {
          span: node.span,
          name: format!("...{}", name),
          pattern: None,
          default_value: None,
        })
      }
      _ => Ok(Param {
        span: node.span,
        name: self.node_text(node),
        pattern: None,
        default_value: None,
      }),
    }
  }

  // ── Pattern lowering ──────────────────────────────────────────────

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
      _ => Ok(Pattern::Object(ObjectPattern {
        span: node.span,
        properties: vec![ObjectPatternProperty::Shorthand(PatternShorthand {
          span: node.span,
          name: self.node_text(node),
          default_value: None,
        })],
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
