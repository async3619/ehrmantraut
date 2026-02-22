use std::fmt;

use ehrmantraut_core::source::{Position, Span};

use crate::cst::CstNode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
  JavaScript,
  TypeScript,
  Tsx,
}

#[derive(Debug)]
pub enum ParseError {
  LanguageSetup(String),
  ParseFailed,
}

impl fmt::Display for ParseError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      ParseError::LanguageSetup(msg) => write!(f, "language setup error: {msg}"),
      ParseError::ParseFailed => write!(f, "parse failed"),
    }
  }
}

impl std::error::Error for ParseError {}

pub fn parse(source: &str, language: Language) -> Result<CstNode, ParseError> {
  let mut parser = tree_sitter::Parser::new();

  let ts_lang = match language {
    Language::JavaScript => tree_sitter_javascript::LANGUAGE.into(),
    Language::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
    Language::Tsx => tree_sitter_typescript::LANGUAGE_TSX.into(),
  };

  parser
    .set_language(&ts_lang)
    .map_err(|e| ParseError::LanguageSetup(e.to_string()))?;

  let tree = parser.parse(source, None).ok_or(ParseError::ParseFailed)?;

  Ok(tree_to_cst(tree.root_node(), source, None))
}

fn tree_to_cst(node: tree_sitter::Node, source: &str, field_name: Option<&str>) -> CstNode {
  let start = node.start_position();
  let end = node.end_position();

  let span = Span {
    start: Position {
      line: u32::try_from(start.row).unwrap_or(u32::MAX),
      column: u32::try_from(start.column).unwrap_or(u32::MAX),
      offset: u32::try_from(node.start_byte()).unwrap_or(u32::MAX),
    },
    end: Position {
      line: u32::try_from(end.row).unwrap_or(u32::MAX),
      column: u32::try_from(end.column).unwrap_or(u32::MAX),
      offset: u32::try_from(node.end_byte()).unwrap_or(u32::MAX),
    },
  };

  let child_count = node.child_count();
  let mut cursor = node.walk();

  let children: Vec<CstNode> = node
    .children(&mut cursor)
    .enumerate()
    .map(|(i, child)| {
      let fname = node.field_name_for_child(i as u32);
      tree_to_cst(child, source, fname)
    })
    .collect();

  let text = if child_count == 0 {
    source
      .get(node.start_byte()..node.end_byte())
      .map(|s| s.to_string())
  } else {
    None
  };

  CstNode {
    kind: node.kind().to_string(),
    named: node.is_named(),
    span,
    text,
    children,
    field_name: field_name.map(|s| s.to_string()),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse_js_variable_declaration() {
    let cst = parse("let x = 1;", Language::JavaScript).unwrap();
    assert_eq!(cst.kind, "program");
    assert!(cst.children.iter().any(|c| c.kind == "lexical_declaration"));
  }

  #[test]
  fn parse_ts_variable_declaration() {
    let cst = parse("let x: number = 1;", Language::TypeScript).unwrap();
    assert_eq!(cst.kind, "program");
    assert!(cst.children.iter().any(|c| c.kind == "lexical_declaration"));
  }

  #[test]
  fn parse_tsx_jsx_element() {
    let cst = parse("const el = <div />;", Language::Tsx).unwrap();
    assert_eq!(cst.kind, "program");
  }

  #[test]
  fn leaf_nodes_have_text() {
    let cst = parse("let x = 1;", Language::JavaScript).unwrap();
    let decl = &cst.children.iter().find(|c| c.named).unwrap();
    // Find the identifier "x" in the tree
    fn find_kind<'a>(node: &'a CstNode, kind: &str) -> Option<&'a CstNode> {
      if node.kind == kind {
        return Some(node);
      }
      node.children.iter().find_map(|c| find_kind(c, kind))
    }
    let ident = find_kind(decl, "identifier").unwrap();
    assert_eq!(ident.text.as_deref(), Some("x"));
    assert!(ident.children.is_empty());
  }
}
