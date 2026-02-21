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

  // Module
  ImportDecl(ImportDecl),
  ExportDecl(ExportDecl),

  // Control flow
  If(IfStmt),
  For(ForStmt),
  While(WhileStmt),
  Switch(SwitchStmt),
  TryCatch(TryCatchStmt),
  Return(ReturnStmt),
  Throw(ThrowStmt),
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
  UnaryExpr(UnaryExpr),
  UpdateExpr(UpdateExpr),
  ConditionalExpr(ConditionalExpr),
  ArrayExpr(ArrayExpr),
  ObjectExpr(ObjectExpr),
  FunctionExpr(FunctionDecl),
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
  pub is_arrow: bool,
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

// ── Module nodes (import / export) ───────────────────────────────────

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../bindings/")]
#[serde(rename_all = "camelCase")]
pub struct ImportDecl {
  pub span: Span,
  pub specifiers: Vec<ImportSpecifier>,
  pub source: String,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../bindings/")]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ImportSpecifier {
  Default(ImportDefault),
  Named(ImportNamed),
  Namespace(ImportNamespace),
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../bindings/")]
#[serde(rename_all = "camelCase")]
pub struct ImportDefault {
  pub span: Span,
  pub local: String,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../bindings/")]
#[serde(rename_all = "camelCase")]
pub struct ImportNamed {
  pub span: Span,
  pub imported: String,
  pub local: String,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../bindings/")]
#[serde(rename_all = "camelCase")]
pub struct ImportNamespace {
  pub span: Span,
  pub local: String,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../bindings/")]
#[serde(rename_all = "camelCase")]
pub struct ExportDecl {
  pub span: Span,
  pub declaration: Option<Box<IrNode>>,
  pub specifiers: Vec<ExportSpecifier>,
  pub source: Option<String>,
  pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../bindings/")]
#[serde(rename_all = "camelCase")]
pub struct ExportSpecifier {
  pub span: Span,
  pub local: String,
  pub exported: String,
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
pub struct ThrowStmt {
  pub span: Span,
  pub argument: IrExpr,
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
  pub scope_level: Option<ScopeLevel>,
  pub declaration_kind: Option<DeclKind>,
  pub is_async: bool,
  pub is_generator: bool,
  pub is_export: bool,
  pub is_default: bool,
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

// ── Unary / Update / Conditional ─────────────────────────────────────

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../bindings/")]
#[serde(rename_all = "camelCase")]
pub struct UnaryExpr {
  pub span: Span,
  pub operator: String,
  pub operand: Box<IrExpr>,
  pub prefix: bool,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../bindings/")]
#[serde(rename_all = "camelCase")]
pub struct UpdateExpr {
  pub span: Span,
  pub operator: String,
  pub operand: Box<IrExpr>,
  pub prefix: bool,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../bindings/")]
#[serde(rename_all = "camelCase")]
pub struct ConditionalExpr {
  pub span: Span,
  pub condition: Box<IrExpr>,
  pub consequent: Box<IrExpr>,
  pub alternate: Box<IrExpr>,
}

// ── Array / Object literals ─────────────────────────────────────────

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../bindings/")]
#[serde(rename_all = "camelCase")]
pub struct ArrayExpr {
  pub span: Span,
  pub elements: Vec<Option<IrExpr>>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../bindings/")]
#[serde(rename_all = "camelCase")]
pub struct ObjectExpr {
  pub span: Span,
  pub properties: Vec<ObjectProperty>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../bindings/")]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ObjectProperty {
  KeyValue(KeyValueProperty),
  Shorthand(ShorthandProperty),
  Method(MethodProperty),
  Accessor(AccessorProperty),
  Spread(SpreadProperty),
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../bindings/")]
#[serde(rename_all = "camelCase")]
pub struct KeyValueProperty {
  pub span: Span,
  pub key: IrExpr,
  pub value: IrExpr,
  pub computed: bool,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../bindings/")]
#[serde(rename_all = "camelCase")]
pub struct ShorthandProperty {
  pub span: Span,
  pub name: String,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../bindings/")]
#[serde(rename_all = "camelCase")]
pub struct MethodProperty {
  pub span: Span,
  pub key: IrExpr,
  pub params: Vec<Param>,
  pub body: Block,
  pub computed: bool,
  pub is_async: bool,
  pub is_generator: bool,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../bindings/")]
#[serde(rename_all = "camelCase")]
pub struct AccessorProperty {
  pub span: Span,
  pub key: IrExpr,
  pub accessor_kind: AccessorKind,
  pub params: Vec<Param>,
  pub body: Block,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../bindings/")]
#[serde(rename_all = "camelCase")]
pub enum AccessorKind {
  Get,
  Set,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../bindings/")]
#[serde(rename_all = "camelCase")]
pub struct SpreadProperty {
  pub span: Span,
  pub argument: IrExpr,
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
