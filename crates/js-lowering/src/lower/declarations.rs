use ehrmantraut_core::ir::{
  Annotations, Block, ClassDecl, DeclKind, FunctionDecl, IrNode, Param, ScopeLevel, VariableDecl,
};

use super::{JsLowerer, LowerError};

impl JsLowerer {
  pub fn lower_variable_declaration(
    &mut self,
    node: &crate::cst::CstNode,
  ) -> Result<IrNode, LowerError> {
    // Determine the declaration kind (let/const/var) from the first unnamed child
    let kind_text = node
      .children
      .iter()
      .find(|c| !c.named && matches!(c.kind.as_str(), "let" | "const" | "var"))
      .map(|c| c.kind.as_str())
      .unwrap_or("");

    let (decl_kind, scope_level) = Self::decl_kind_and_scope(kind_text);

    // Find all variable_declarator children
    let declarators: Vec<_> = node
      .children
      .iter()
      .filter(|c| c.kind == "variable_declarator")
      .collect();

    // For simplicity, lower the first declarator.
    // Multi-declarator statements (let a = 1, b = 2) produce a single
    // VariableDecl for the first declarator — this is intentional for the
    // initial implementation and can be expanded later.
    if let Some(declarator) = declarators.first() {
      let name = self
        .child_by_field(declarator, "name")
        .map(|n| self.node_text(n))
        .unwrap_or_default();

      let value = self
        .child_by_field(declarator, "value")
        .map(|v| self.lower_expression(v))
        .transpose()?;

      Ok(IrNode::VariableDecl(VariableDecl {
        span: node.span,
        name,
        value,
        annotations: Annotations {
          scope_level,
          declaration_kind: decl_kind,
          ..Default::default()
        },
      }))
    } else {
      Ok(IrNode::VariableDecl(VariableDecl {
        span: node.span,
        name: String::new(),
        value: None,
        annotations: Annotations {
          scope_level,
          declaration_kind: decl_kind,
          ..Default::default()
        },
      }))
    }
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

    // Check for async/generator from unnamed children
    let is_async = node.children.iter().any(|c| c.kind == "async");
    let is_generator = node
      .children
      .iter()
      .any(|c| c.kind == "*" || c.kind == "generator_function_declaration");

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
        // tree-sitter uses "class_heritage" for extends
        node
          .children
          .iter()
          .find(|c| c.kind == "class_heritage")
          .and_then(|h| self.first_named_child(h))
      });

    let super_class_expr = super_class.map(|s| self.lower_expression(s)).transpose()?;

    // Lower class body
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
