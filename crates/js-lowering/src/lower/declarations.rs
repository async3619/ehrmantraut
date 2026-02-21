use ehrmantraut_core::ir::{
  Annotations, Block, ClassDecl, DeclKind, ExportDecl, ExportSpecifier, FunctionDecl, ImportDecl,
  ImportDefault, ImportNamed, ImportNamespace, ImportSpecifier, IrNode, Param, ScopeLevel,
  VariableDecl,
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
      let name = self
        .child_by_field(declarator, "name")
        .map(|n| self.node_text(n))
        .unwrap_or_default();

      let value = self
        .child_by_field(declarator, "value")
        .map(|v| self.lower_expression(v))
        .transpose()?;

      nodes.push(IrNode::VariableDecl(VariableDecl {
        span: declarator.span,
        name,
        value,
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
      .map(|s| {
        let text = self.node_text(s);
        // Strip quotes
        if (text.starts_with('"') && text.ends_with('"'))
          || (text.starts_with('\'') && text.ends_with('\''))
        {
          text[1..text.len() - 1].to_string()
        } else {
          text
        }
      })
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

    let source = self.child_by_field(node, "source").map(|s| {
      let text = self.node_text(s);
      if (text.starts_with('"') && text.ends_with('"'))
        || (text.starts_with('\'') && text.ends_with('\''))
      {
        text[1..text.len() - 1].to_string()
      } else {
        text
      }
    });

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

  fn lower_formal_parameters(&mut self, func_node: &crate::cst::CstNode) -> Vec<Param> {
    let params_node = self.child_by_field(func_node, "parameters");
    match params_node {
      Some(p) => p
        .children
        .iter()
        .filter(|c| c.named)
        .map(|c| Param {
          span: c.span,
          name: self.node_text(c),
        })
        .collect(),
      None => Vec::new(),
    }
  }
}
