use ehrmantraut_core::ir::{
  ExportDecl, ExportSpecifier, ImportDecl, ImportDefault, ImportNamed, ImportNamespace,
  ImportSpecifier, IrNode,
};

use crate::lower::{JsLowerer, LowerError};

impl JsLowerer {
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
}
