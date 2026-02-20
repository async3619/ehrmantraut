use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Span {
  pub start: Position,
  pub end: Position,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Position {
  /// 0-based line number
  pub line: u32,
  /// 0-based column (byte offset within the line)
  pub column: u32,
  /// Byte offset from the start of the source
  pub offset: u32,
}
