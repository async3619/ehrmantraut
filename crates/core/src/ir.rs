use serde::Serialize;
use ts_rs::TS;

use crate::source::Span;

// ── Module (top-level) ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../bindings/")]
#[serde(rename_all = "camelCase")]
pub struct IrModule {
  pub span: Span,
  pub body: Vec<IrNode>,
}

// ── Statements / Nodes ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../bindings/")]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum IrNode {
  // Declarations
  VariableDecl(VariableDecl),
  FunctionDecl(FunctionDecl),
  ClassDecl(ClassDecl),

  // Control flow
  If(IfStmt),
  For(ForStmt),
  While(WhileStmt),
  Switch(SwitchStmt),
  TryCatch(TryCatchStmt),
  Return(ReturnStmt),
  Break(BreakStmt),
  Continue(ContinueStmt),

  // Expressions used as statements
  ExpressionStatement(ExpressionStatement),

  // Scoping
  Block(Block),

  // Catch-all for statements we don't lower in detail
  Opaque(OpaqueNode),
}

// ── Expressions ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../bindings/")]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum IrExpr {
  Assignment(Assignment),
  Call(Call),
  MemberAccess(MemberAccess),
  BinaryExpr(BinaryExpr),
  Identifier(Identifier),
  Literal(Literal),
  // Catch-all for expressions we don't lower in detail
  Opaque(OpaqueExpr),
}

// ── Declaration nodes ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../bindings/")]
#[serde(rename_all = "camelCase")]
pub struct VariableDecl {
  pub span: Span,
  pub name: String,
  pub value: Option<IrExpr>,
  pub annotations: Annotations,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../bindings/")]
#[serde(rename_all = "camelCase")]
pub struct FunctionDecl {
  pub span: Span,
  pub name: Option<String>,
  pub params: Vec<Param>,
  pub body: Block,
  pub annotations: Annotations,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../bindings/")]
#[serde(rename_all = "camelCase")]
pub struct Param {
  pub span: Span,
  pub name: String,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../bindings/")]
#[serde(rename_all = "camelCase")]
pub struct ClassDecl {
  pub span: Span,
  pub name: Option<String>,
  pub super_class: Option<IrExpr>,
  pub body: Vec<IrNode>,
  pub annotations: Annotations,
}

// ── Control flow nodes ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../bindings/")]
#[serde(rename_all = "camelCase")]
pub struct IfStmt {
  pub span: Span,
  pub condition: IrExpr,
  pub consequent: Block,
  pub alternate: Option<Box<IrNode>>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../bindings/")]
#[serde(rename_all = "camelCase")]
pub struct ForStmt {
  pub span: Span,
  pub init: Option<Box<IrNode>>,
  pub condition: Option<IrExpr>,
  pub update: Option<IrExpr>,
  pub body: Block,
  pub annotations: ForAnnotations,
}

#[derive(Debug, Clone, Default, Serialize, TS)]
#[ts(export, export_to = "../../bindings/")]
#[serde(rename_all = "camelCase")]
pub struct ForAnnotations {
  pub kind: ForKind,
}

#[derive(Debug, Clone, Default, Serialize, TS)]
#[ts(export, export_to = "../../bindings/")]
#[serde(rename_all = "camelCase")]
pub enum ForKind {
  #[default]
  Standard,
  In,
  Of,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../bindings/")]
#[serde(rename_all = "camelCase")]
pub struct WhileStmt {
  pub span: Span,
  pub condition: IrExpr,
  pub body: Block,
  pub is_do_while: bool,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../bindings/")]
#[serde(rename_all = "camelCase")]
pub struct SwitchStmt {
  pub span: Span,
  pub discriminant: IrExpr,
  pub cases: Vec<SwitchCase>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../bindings/")]
#[serde(rename_all = "camelCase")]
pub struct SwitchCase {
  pub span: Span,
  pub test: Option<IrExpr>,
  pub body: Vec<IrNode>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../bindings/")]
#[serde(rename_all = "camelCase")]
pub struct TryCatchStmt {
  pub span: Span,
  pub try_block: Block,
  pub catch_clause: Option<CatchClause>,
  pub finally_block: Option<Block>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../bindings/")]
#[serde(rename_all = "camelCase")]
pub struct CatchClause {
  pub span: Span,
  pub param: Option<String>,
  pub body: Block,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../bindings/")]
#[serde(rename_all = "camelCase")]
pub struct ReturnStmt {
  pub span: Span,
  pub value: Option<IrExpr>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../bindings/")]
#[serde(rename_all = "camelCase")]
pub struct BreakStmt {
  pub span: Span,
  pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../bindings/")]
#[serde(rename_all = "camelCase")]
pub struct ContinueStmt {
  pub span: Span,
  pub label: Option<String>,
}

// ── Expression nodes ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../bindings/")]
#[serde(rename_all = "camelCase")]
pub struct ExpressionStatement {
  pub span: Span,
  pub expression: IrExpr,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../bindings/")]
#[serde(rename_all = "camelCase")]
pub struct Assignment {
  pub span: Span,
  pub target: Box<IrExpr>,
  pub operator: String,
  pub value: Box<IrExpr>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../bindings/")]
#[serde(rename_all = "camelCase")]
pub struct Call {
  pub span: Span,
  pub callee: Box<IrExpr>,
  pub arguments: Vec<IrExpr>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../bindings/")]
#[serde(rename_all = "camelCase")]
pub struct MemberAccess {
  pub span: Span,
  pub object: Box<IrExpr>,
  pub property: Box<IrExpr>,
  pub computed: bool,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../bindings/")]
#[serde(rename_all = "camelCase")]
pub struct BinaryExpr {
  pub span: Span,
  pub left: Box<IrExpr>,
  pub operator: String,
  pub right: Box<IrExpr>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../bindings/")]
#[serde(rename_all = "camelCase")]
pub struct Identifier {
  pub span: Span,
  pub name: String,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../bindings/")]
#[serde(rename_all = "camelCase")]
pub struct Literal {
  pub span: Span,
  pub value: LiteralValue,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../bindings/")]
#[serde(rename_all = "camelCase")]
pub enum LiteralValue {
  String(String),
  Number(f64),
  Boolean(bool),
  Null,
  Undefined,
}

// ── Scoping ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../bindings/")]
#[serde(rename_all = "camelCase")]
pub struct Block {
  pub span: Span,
  pub body: Vec<IrNode>,
}

// ── Annotations ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, Serialize, TS)]
#[ts(export, export_to = "../../bindings/")]
#[serde(rename_all = "camelCase")]
pub struct Annotations {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub scope_level: Option<ScopeLevel>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub declaration_kind: Option<DeclKind>,
  #[serde(skip_serializing_if = "is_false")]
  pub is_async: bool,
  #[serde(skip_serializing_if = "is_false")]
  pub is_generator: bool,
  #[serde(skip_serializing_if = "is_false")]
  pub is_export: bool,
  #[serde(skip_serializing_if = "is_false")]
  pub is_default: bool,
}

fn is_false(v: &bool) -> bool {
  !v
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[ts(export, export_to = "../../bindings/")]
#[serde(rename_all = "camelCase")]
pub enum ScopeLevel {
  Block,
  Function,
  Module,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[ts(export, export_to = "../../bindings/")]
#[serde(rename_all = "camelCase")]
pub enum DeclKind {
  Let,
  Const,
  Var,
  Function,
  Class,
  Import,
}

// ── Opaque (escape hatch) ────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../bindings/")]
#[serde(rename_all = "camelCase")]
pub struct OpaqueExpr {
  pub span: Span,
  pub cst_kind: String,
  pub text: String,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../bindings/")]
#[serde(rename_all = "camelCase")]
pub struct OpaqueNode {
  pub span: Span,
  pub cst_kind: String,
  pub text: String,
}
