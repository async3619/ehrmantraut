use ehrmantraut_core::ir::{
  Accessibility, Annotations, Block, ClassDecl, DeclKind, IrExpr, IrNode, MethodDefinition,
  MethodKind, OpaqueExpr, PropertyDefinition,
};

use crate::lower::{JsLowerer, LowerError};

impl JsLowerer {
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
      None => IrExpr::Opaque(OpaqueExpr {
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
      None => IrExpr::Opaque(OpaqueExpr {
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

  pub(crate) fn extract_accessibility(node: &crate::cst::CstNode) -> Option<Accessibility> {
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
}
