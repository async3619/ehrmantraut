import test from 'ava'

import { lower } from '../index'

test('lower JS let declaration', (t) => {
  const ir = lower('let x = 1;', 'javascript')
  t.truthy(ir.body)
  t.is(ir.body.length, 1)

  const node = ir.body[0]
  t.is(node.type, 'variableDecl')
  if (node.type !== 'variableDecl') return
  t.is(node.name, 'x')
  t.is(node.annotations.declarationKind, 'let')
  t.is(node.annotations.scopeLevel, 'block')
})

test('lower JS var declaration has function scope level', (t) => {
  const node = lower('var x = 1;', 'javascript').body[0]
  if (node.type !== 'variableDecl') return t.fail()
  t.is(node.annotations.declarationKind, 'var')
  t.is(node.annotations.scopeLevel, 'function')
})

test('lower JS const declaration', (t) => {
  const node = lower('const x = 1;', 'javascript').body[0]
  if (node.type !== 'variableDecl') return t.fail()
  t.is(node.annotations.declarationKind, 'const')
  t.is(node.annotations.scopeLevel, 'block')
})

test('lower JS function declaration', (t) => {
  const node = lower('function foo(a, b) { return a + b; }', 'javascript').body[0]
  if (node.type !== 'functionDecl') return t.fail()
  t.is(node.name, 'foo')
  t.is(node.params.length, 2)
})

test('lower JS if statement', (t) => {
  const node = lower('if (x) { y; }', 'javascript').body[0]
  if (node.type !== 'if') return t.fail()
  t.truthy(node.condition)
  t.truthy(node.consequent)
})

test('lower JS for statement', (t) => {
  const node = lower('for (let i = 0; i < 10; i++) { x; }', 'javascript').body[0]
  if (node.type !== 'for') return t.fail()
  t.truthy(node.body)
})

test('lower JS while statement', (t) => {
  const node = lower('while (true) { x; }', 'javascript').body[0]
  if (node.type !== 'while') return t.fail()
  t.is(node.isDoWhile, false)
})

test('lower JS try-catch statement', (t) => {
  const node = lower('try { x; } catch (e) { y; } finally { z; }', 'javascript').body[0]
  if (node.type !== 'tryCatch') return t.fail()
  t.truthy(node.tryBlock)
  t.truthy(node.catchClause)
  t.truthy(node.finallyBlock)
})

test('lower JS expression statement with call', (t) => {
  const node = lower('console.log("hello");', 'javascript').body[0]
  if (node.type !== 'expressionStatement') return t.fail()
  t.is(node.expression.type, 'call')
})

test('lower TS ignores type annotations', (t) => {
  const node = lower('let x: number = 1;', 'typescript').body[0]
  if (node.type !== 'variableDecl') return t.fail()
  t.is(node.name, 'x')
})

test('lower preserves source span', (t) => {
  const ir = lower('let x = 1;', 'javascript')
  t.is(ir.span.start.line, 0)
  t.is(ir.span.start.offset, 0)
  t.truthy(ir.span.end.offset > 0)
})

test('lower multiple statements', (t) => {
  const ir = lower('let x = 1;\nlet y = 2;', 'javascript')
  t.is(ir.body.length, 2)

  const first = ir.body[0]
  const second = ir.body[1]
  if (first.type !== 'variableDecl' || second.type !== 'variableDecl') return t.fail()
  t.is(first.name, 'x')
  t.is(second.name, 'y')
})

test('lower JS multi-declarator splits into separate nodes', (t) => {
  const ir = lower('let a = 1, b = 2, c = 3;', 'javascript')
  t.is(ir.body.length, 3)

  for (const node of ir.body) {
    if (node.type !== 'variableDecl') return t.fail()
    t.is(node.annotations.declarationKind, 'let')
    t.is(node.annotations.scopeLevel, 'block')
  }

  const [a, b, c] = ir.body
  if (a.type !== 'variableDecl' || b.type !== 'variableDecl' || c.type !== 'variableDecl') return t.fail()
  t.is(a.name, 'a')
  t.is(b.name, 'b')
  t.is(c.name, 'c')
})

// ── break / continue ────────────────────────────────────────────────

test('lower JS break statement', (t) => {
  const node = lower('while (true) { break; }', 'javascript').body[0]
  if (node.type !== 'while') return t.fail()
  const stmt = node.body.body[0]
  if (stmt.type !== 'break') return t.fail()
  t.is(stmt.label, null)
})

test('lower JS labeled break statement', (t) => {
  const node = lower('outer: while (true) { break outer; }', 'javascript').body[0]
  if (node.type !== 'opaque') return // labeled statement wraps
  t.pass()
})

test('lower JS continue statement', (t) => {
  const node = lower('while (true) { continue; }', 'javascript').body[0]
  if (node.type !== 'while') return t.fail()
  const stmt = node.body.body[0]
  if (stmt.type !== 'continue') return t.fail()
  t.is(stmt.label, null)
})

// ── expressions ─────────────────────────────────────────────────────

test('lower JS assignment expression', (t) => {
  const node = lower('x = 1;', 'javascript').body[0]
  if (node.type !== 'expressionStatement') return t.fail()
  if (node.expression.type !== 'assignment') return t.fail()
  t.is(node.expression.operator, '=')
})

test('lower JS compound assignment expression', (t) => {
  const node = lower('x += 1;', 'javascript').body[0]
  if (node.type !== 'expressionStatement') return t.fail()
  if (node.expression.type !== 'assignment') return t.fail()
  t.is(node.expression.operator, '+=')
})

test('lower JS binary expression', (t) => {
  const node = lower('x + y;', 'javascript').body[0]
  if (node.type !== 'expressionStatement') return t.fail()
  if (node.expression.type !== 'binaryExpr') return t.fail()
  t.is(node.expression.operator, '+')
})

test('lower JS member access (dot notation)', (t) => {
  const node = lower('a.b;', 'javascript').body[0]
  if (node.type !== 'expressionStatement') return t.fail()
  if (node.expression.type !== 'memberAccess') return t.fail()
  t.is(node.expression.computed, false)
})

test('lower JS member access (bracket notation)', (t) => {
  const node = lower('a[0];', 'javascript').body[0]
  if (node.type !== 'expressionStatement') return t.fail()
  if (node.expression.type !== 'memberAccess') return t.fail()
  t.is(node.expression.computed, true)
})

test('lower JS number literal', (t) => {
  const node = lower('42;', 'javascript').body[0]
  if (node.type !== 'expressionStatement') return t.fail()
  if (node.expression.type !== 'literal') return t.fail()
  t.deepEqual(node.expression.value, { number: 42 })
})

test('lower JS string literal', (t) => {
  const node = lower('"hello";', 'javascript').body[0]
  if (node.type !== 'expressionStatement') return t.fail()
  if (node.expression.type !== 'literal') return t.fail()
  t.deepEqual(node.expression.value, { string: 'hello' })
})

test('lower JS boolean literal', (t) => {
  const node = lower('true;', 'javascript').body[0]
  if (node.type !== 'expressionStatement') return t.fail()
  if (node.expression.type !== 'literal') return t.fail()
  t.deepEqual(node.expression.value, { boolean: true })
})

test('lower JS null literal', (t) => {
  const node = lower('null;', 'javascript').body[0]
  if (node.type !== 'expressionStatement') return t.fail()
  if (node.expression.type !== 'literal') return t.fail()
  t.is(node.expression.value, 'null')
})

// ── for..in / for..of ───────────────────────────────────────────────

test('lower JS for..in statement', (t) => {
  const node = lower('for (const x in obj) { y; }', 'javascript').body[0]
  if (node.type !== 'for') return t.fail()
  t.is(node.annotations.kind, 'in')
})

test('lower JS for..of statement', (t) => {
  const node = lower('for (const x of arr) { y; }', 'javascript').body[0]
  if (node.type !== 'for') return t.fail()
  t.is(node.annotations.kind, 'of')
})

// ── switch ──────────────────────────────────────────────────────────

test('lower JS switch statement', (t) => {
  const node = lower('switch (x) { case 1: y; break; default: z; }', 'javascript').body[0]
  if (node.type !== 'switch') return t.fail()
  t.is(node.cases.length, 2)
  t.truthy(node.cases[0].test)
  t.is(node.cases[1].test, null)
})

// ── do..while ───────────────────────────────────────────────────────

test('lower JS do..while statement', (t) => {
  const node = lower('do { x; } while (true);', 'javascript').body[0]
  if (node.type !== 'while') return t.fail()
  t.is(node.isDoWhile, true)
})

// ── class ───────────────────────────────────────────────────────────

test('lower JS class declaration', (t) => {
  const node = lower('class Foo {}', 'javascript').body[0]
  if (node.type !== 'classDecl') return t.fail()
  t.is(node.name, 'Foo')
  t.is(node.superClass, null)
})

test('lower JS class declaration with extends', (t) => {
  const node = lower('class Bar extends Foo {}', 'javascript').body[0]
  if (node.type !== 'classDecl') return t.fail()
  t.is(node.name, 'Bar')
  t.truthy(node.superClass)
})

// ── import declarations ────────────────────────────────────────────

test('lower JS default import', (t) => {
  const node = lower("import x from 'mod';", 'javascript').body[0]
  if (node.type !== 'importDecl') return t.fail()
  t.is(node.source, 'mod')
  t.is(node.specifiers.length, 1)
  t.is(node.specifiers[0].kind, 'default')
  if (node.specifiers[0].kind !== 'default') return t.fail()
  t.is(node.specifiers[0].local, 'x')
})

test('lower JS named imports', (t) => {
  const node = lower("import { foo, bar as b } from 'mod';", 'javascript').body[0]
  if (node.type !== 'importDecl') return t.fail()
  t.is(node.specifiers.length, 2)
  if (node.specifiers[0].kind !== 'named') return t.fail()
  t.is(node.specifiers[0].imported, 'foo')
  t.is(node.specifiers[0].local, 'foo')
  if (node.specifiers[1].kind !== 'named') return t.fail()
  t.is(node.specifiers[1].imported, 'bar')
  t.is(node.specifiers[1].local, 'b')
})

test('lower JS namespace import', (t) => {
  const node = lower("import * as ns from 'mod';", 'javascript').body[0]
  if (node.type !== 'importDecl') return t.fail()
  t.is(node.specifiers.length, 1)
  if (node.specifiers[0].kind !== 'namespace') return t.fail()
  t.is(node.specifiers[0].local, 'ns')
})

test('lower JS side-effect import', (t) => {
  const node = lower("import 'mod';", 'javascript').body[0]
  if (node.type !== 'importDecl') return t.fail()
  t.is(node.specifiers.length, 0)
  t.is(node.source, 'mod')
})

// ── export declarations ────────────────────────────────────────────

test('lower JS export function declaration', (t) => {
  const node = lower('export function foo() {}', 'javascript').body[0]
  if (node.type !== 'functionDecl') return t.fail()
  t.is(node.annotations.isExport, true)
  t.is(node.name, 'foo')
})

test('lower JS export const', (t) => {
  const node = lower('export const x = 1;', 'javascript').body[0]
  if (node.type !== 'variableDecl') return t.fail()
  t.is(node.annotations.isExport, true)
  t.is(node.name, 'x')
})

test('lower JS named exports', (t) => {
  const node = lower('export { x, y as z };', 'javascript').body[0]
  if (node.type !== 'exportDecl') return t.fail()
  t.is(node.specifiers.length, 2)
  t.is(node.specifiers[0].local, 'x')
  t.is(node.specifiers[0].exported, 'x')
  t.is(node.specifiers[1].local, 'y')
  t.is(node.specifiers[1].exported, 'z')
})

test('lower JS export default expression', (t) => {
  const node = lower('export default 42;', 'javascript').body[0]
  if (node.type !== 'exportDecl') return t.fail()
  t.is(node.isDefault, true)
  t.truthy(node.declaration)
})

test('lower JS re-export', (t) => {
  const node = lower("export { x } from 'mod';", 'javascript').body[0]
  if (node.type !== 'exportDecl') return t.fail()
  t.is(node.source, 'mod')
  t.is(node.specifiers.length, 1)
})

test('lower JS namespace re-export', (t) => {
  const node = lower("export * from 'mod';", 'javascript').body[0]
  if (node.type !== 'exportDecl') return t.fail()
  t.is(node.source, 'mod')
  t.is(node.specifiers[0].local, '*')
})

// ── arrow function expression ──────────────────────────────────────

test('lower JS arrow function concise body', (t) => {
  const node = lower('const fn = (x) => x + 1;', 'javascript').body[0]
  if (node.type !== 'variableDecl') return t.fail()
  if (node.value?.type !== 'functionExpr') return t.fail()
  t.is(node.value.isArrow, true)
  t.is(node.value.params.length, 1)
  t.is(node.value.params[0].name, 'x')
  // Concise body should be wrapped in implicit return
  t.is(node.value.body.body.length, 1)
  t.is(node.value.body.body[0].type, 'return')
})

test('lower JS arrow function block body', (t) => {
  const node = lower('const fn = () => { return 1 };', 'javascript').body[0]
  if (node.type !== 'variableDecl') return t.fail()
  if (node.value?.type !== 'functionExpr') return t.fail()
  t.is(node.value.isArrow, true)
  t.is(node.value.body.body.length, 1)
  t.is(node.value.body.body[0].type, 'return')
})

test('lower JS async arrow function', (t) => {
  const node = lower('const fn = async (x) => x;', 'javascript').body[0]
  if (node.type !== 'variableDecl') return t.fail()
  if (node.value?.type !== 'functionExpr') return t.fail()
  t.is(node.value.isArrow, true)
  t.is(node.value.annotations.isAsync, true)
})

test('lower JS arrow function single param no parens', (t) => {
  const node = lower('const fn = x => x;', 'javascript').body[0]
  if (node.type !== 'variableDecl') return t.fail()
  if (node.value?.type !== 'functionExpr') return t.fail()
  t.is(node.value.params.length, 1)
  t.is(node.value.params[0].name, 'x')
})

test('lower JS function expression', (t) => {
  const node = lower('const fn = function foo(a) { return a };', 'javascript').body[0]
  if (node.type !== 'variableDecl') return t.fail()
  if (node.value?.type !== 'functionExpr') return t.fail()
  t.is(node.value.isArrow, false)
  t.is(node.value.name, 'foo')
  t.is(node.value.params.length, 1)
})

test('lower JS function declaration has isArrow false', (t) => {
  const node = lower('function foo() {}', 'javascript').body[0]
  if (node.type !== 'functionDecl') return t.fail()
  t.is(node.isArrow, false)
})

// ── throw statement ────────────────────────────────────────────────

test('lower JS throw statement', (t) => {
  const node = lower('throw new Error("msg");', 'javascript').body[0]
  if (node.type !== 'throw') return t.fail()
  t.is(node.argument.type, 'opaque') // new expression is still opaque
})

test('lower JS throw with expression', (t) => {
  const node = lower('throw x;', 'javascript').body[0]
  if (node.type !== 'throw') return t.fail()
  if (node.argument.type !== 'identifier') return t.fail()
  t.is(node.argument.name, 'x')
})

// ── unary expressions ──────────────────────────────────────────────

test('lower JS unary not expression', (t) => {
  const node = lower('!x;', 'javascript').body[0]
  if (node.type !== 'expressionStatement') return t.fail()
  if (node.expression.type !== 'unaryExpr') return t.fail()
  t.is(node.expression.operator, '!')
  t.is(node.expression.prefix, true)
  if (node.expression.operand.type !== 'identifier') return t.fail()
  t.is(node.expression.operand.name, 'x')
})

test('lower JS typeof expression', (t) => {
  const node = lower('typeof x;', 'javascript').body[0]
  if (node.type !== 'expressionStatement') return t.fail()
  if (node.expression.type !== 'unaryExpr') return t.fail()
  t.is(node.expression.operator, 'typeof')
  t.is(node.expression.prefix, true)
})

test('lower JS void expression', (t) => {
  const node = lower('void 0;', 'javascript').body[0]
  if (node.type !== 'expressionStatement') return t.fail()
  if (node.expression.type !== 'unaryExpr') return t.fail()
  t.is(node.expression.operator, 'void')
})

test('lower JS delete expression', (t) => {
  const node = lower('delete obj.key;', 'javascript').body[0]
  if (node.type !== 'expressionStatement') return t.fail()
  if (node.expression.type !== 'unaryExpr') return t.fail()
  t.is(node.expression.operator, 'delete')
})

test('lower JS unary minus expression', (t) => {
  const node = lower('-x;', 'javascript').body[0]
  if (node.type !== 'expressionStatement') return t.fail()
  if (node.expression.type !== 'unaryExpr') return t.fail()
  t.is(node.expression.operator, '-')
})

// ── update expressions ─────────────────────────────────────────────

test('lower JS prefix increment', (t) => {
  const node = lower('++x;', 'javascript').body[0]
  if (node.type !== 'expressionStatement') return t.fail()
  if (node.expression.type !== 'updateExpr') return t.fail()
  t.is(node.expression.operator, '++')
  t.is(node.expression.prefix, true)
})

test('lower JS postfix increment', (t) => {
  const node = lower('x++;', 'javascript').body[0]
  if (node.type !== 'expressionStatement') return t.fail()
  if (node.expression.type !== 'updateExpr') return t.fail()
  t.is(node.expression.operator, '++')
  t.is(node.expression.prefix, false)
})

test('lower JS prefix decrement', (t) => {
  const node = lower('--x;', 'javascript').body[0]
  if (node.type !== 'expressionStatement') return t.fail()
  if (node.expression.type !== 'updateExpr') return t.fail()
  t.is(node.expression.operator, '--')
  t.is(node.expression.prefix, true)
})

test('lower JS postfix decrement', (t) => {
  const node = lower('x--;', 'javascript').body[0]
  if (node.type !== 'expressionStatement') return t.fail()
  if (node.expression.type !== 'updateExpr') return t.fail()
  t.is(node.expression.operator, '--')
  t.is(node.expression.prefix, false)
})

// ── conditional (ternary) expression ───────────────────────────────

test('lower JS ternary expression', (t) => {
  const node = lower('a ? b : c;', 'javascript').body[0]
  if (node.type !== 'expressionStatement') return t.fail()
  if (node.expression.type !== 'conditionalExpr') return t.fail()
  if (node.expression.condition.type !== 'identifier') return t.fail()
  t.is(node.expression.condition.name, 'a')
  if (node.expression.consequent.type !== 'identifier') return t.fail()
  t.is(node.expression.consequent.name, 'b')
  if (node.expression.alternate.type !== 'identifier') return t.fail()
  t.is(node.expression.alternate.name, 'c')
})

test('lower JS nested ternary expression', (t) => {
  const node = lower('a ? b : c ? d : e;', 'javascript').body[0]
  if (node.type !== 'expressionStatement') return t.fail()
  if (node.expression.type !== 'conditionalExpr') return t.fail()
  t.is(node.expression.alternate.type, 'conditionalExpr')
})

// ── array expression ───────────────────────────────────────────────

test('lower JS basic array expression', (t) => {
  const node = lower('[1, 2, 3];', 'javascript').body[0]
  if (node.type !== 'expressionStatement') return t.fail()
  if (node.expression.type !== 'arrayExpr') return t.fail()
  t.is(node.expression.elements.length, 3)
  for (const elem of node.expression.elements) {
    if (!elem) return t.fail()
    t.is(elem.type, 'literal')
  }
})

test('lower JS sparse array (holes)', (t) => {
  const node = lower('[1, , 3];', 'javascript').body[0]
  if (node.type !== 'expressionStatement') return t.fail()
  if (node.expression.type !== 'arrayExpr') return t.fail()
  t.is(node.expression.elements.length, 3)
  t.truthy(node.expression.elements[0])
  t.is(node.expression.elements[1], null)
  t.truthy(node.expression.elements[2])
})

test('lower JS empty array', (t) => {
  const node = lower('[];', 'javascript').body[0]
  if (node.type !== 'expressionStatement') return t.fail()
  if (node.expression.type !== 'arrayExpr') return t.fail()
  t.is(node.expression.elements.length, 0)
})

test('lower JS array with nested expressions', (t) => {
  const node = lower('[a + b, fn()];', 'javascript').body[0]
  if (node.type !== 'expressionStatement') return t.fail()
  if (node.expression.type !== 'arrayExpr') return t.fail()
  t.is(node.expression.elements.length, 2)
  const first = node.expression.elements[0]
  const second = node.expression.elements[1]
  if (!first || !second) return t.fail()
  t.is(first.type, 'binaryExpr')
  t.is(second.type, 'call')
})

// ── object expression ──────────────────────────────────────────────

test('lower JS basic object expression', (t) => {
  const node = lower('({a: 1, b: 2});', 'javascript').body[0]
  if (node.type !== 'expressionStatement') return t.fail()
  if (node.expression.type !== 'objectExpr') return t.fail()
  t.is(node.expression.properties.length, 2)
  t.is(node.expression.properties[0].kind, 'keyValue')
  t.is(node.expression.properties[1].kind, 'keyValue')
})

test('lower JS object shorthand property', (t) => {
  const node = lower('({x, y});', 'javascript').body[0]
  if (node.type !== 'expressionStatement') return t.fail()
  if (node.expression.type !== 'objectExpr') return t.fail()
  t.is(node.expression.properties.length, 2)
  t.is(node.expression.properties[0].kind, 'shorthand')
  if (node.expression.properties[0].kind !== 'shorthand') return t.fail()
  t.is(node.expression.properties[0].name, 'x')
})

test('lower JS object computed property', (t) => {
  const node = lower('({["a"]: 1});', 'javascript').body[0]
  if (node.type !== 'expressionStatement') return t.fail()
  if (node.expression.type !== 'objectExpr') return t.fail()
  if (node.expression.properties[0].kind !== 'keyValue') return t.fail()
  t.is(node.expression.properties[0].computed, true)
})

test('lower JS object method shorthand', (t) => {
  const node = lower('({foo() { return 1 }});', 'javascript').body[0]
  if (node.type !== 'expressionStatement') return t.fail()
  if (node.expression.type !== 'objectExpr') return t.fail()
  t.is(node.expression.properties[0].kind, 'method')
  if (node.expression.properties[0].kind !== 'method') return t.fail()
  if (node.expression.properties[0].key.type !== 'identifier') return t.fail()
  t.is(node.expression.properties[0].key.name, 'foo')
})

test('lower JS object getter and setter', (t) => {
  const node = lower('({get x() { return 1 }, set x(v) { }});', 'javascript').body[0]
  if (node.type !== 'expressionStatement') return t.fail()
  if (node.expression.type !== 'objectExpr') return t.fail()
  t.is(node.expression.properties.length, 2)
  if (node.expression.properties[0].kind !== 'accessor') return t.fail()
  t.is(node.expression.properties[0].accessorKind, 'get')
  if (node.expression.properties[1].kind !== 'accessor') return t.fail()
  t.is(node.expression.properties[1].accessorKind, 'set')
})

test('lower JS empty object', (t) => {
  const node = lower('({});', 'javascript').body[0]
  if (node.type !== 'expressionStatement') return t.fail()
  if (node.expression.type !== 'objectExpr') return t.fail()
  t.is(node.expression.properties.length, 0)
})

// ── destructuring patterns ─────────────────────────────────────────

test('lower JS object destructuring', (t) => {
  const node = lower('const { x, y } = obj;', 'javascript').body[0]
  if (node.type !== 'variableDecl') return t.fail()
  t.is(node.name, '')
  t.truthy(node.pattern)
  if (node.pattern?.kind !== 'object') return t.fail()
  t.is(node.pattern.properties.length, 2)
  t.is(node.pattern.properties[0].kind, 'shorthand')
  if (node.pattern.properties[0].kind !== 'shorthand') return t.fail()
  t.is(node.pattern.properties[0].name, 'x')
})

test('lower JS array destructuring', (t) => {
  const node = lower('const [a, b] = arr;', 'javascript').body[0]
  if (node.type !== 'variableDecl') return t.fail()
  t.truthy(node.pattern)
  if (node.pattern?.kind !== 'array') return t.fail()
  t.is(node.pattern.elements.length, 2)
})

test('lower JS object destructuring with default', (t) => {
  const node = lower('const { x = 10 } = obj;', 'javascript').body[0]
  if (node.type !== 'variableDecl') return t.fail()
  if (node.pattern?.kind !== 'object') return t.fail()
  if (node.pattern.properties[0].kind !== 'shorthand') return t.fail()
  t.truthy(node.pattern.properties[0].defaultValue)
})

test('lower JS object destructuring with rename', (t) => {
  const node = lower('const { x: renamed } = obj;', 'javascript').body[0]
  if (node.type !== 'variableDecl') return t.fail()
  if (node.pattern?.kind !== 'object') return t.fail()
  t.is(node.pattern.properties[0].kind, 'keyValue')
  if (node.pattern.properties[0].kind !== 'keyValue') return t.fail()
  t.is(node.pattern.properties[0].key, 'x')
})

test('lower JS object destructuring with rest', (t) => {
  const node = lower('const { a, ...rest } = obj;', 'javascript').body[0]
  if (node.type !== 'variableDecl') return t.fail()
  if (node.pattern?.kind !== 'object') return t.fail()
  t.is(node.pattern.properties.length, 2)
  t.is(node.pattern.properties[1].kind, 'rest')
  if (node.pattern.properties[1].kind !== 'rest') return t.fail()
  t.is(node.pattern.properties[1].name, 'rest')
})

test('lower JS function param destructuring', (t) => {
  const node = lower('function fn({ x, y }) {}', 'javascript').body[0]
  if (node.type !== 'functionDecl') return t.fail()
  t.is(node.params[0].name, '')
  t.truthy(node.params[0].pattern)
  if (node.params[0].pattern?.kind !== 'object') return t.fail()
  t.is(node.params[0].pattern.properties.length, 2)
})

test('lower JS simple variable has no pattern', (t) => {
  const node = lower('const x = 1;', 'javascript').body[0]
  if (node.type !== 'variableDecl') return t.fail()
  t.is(node.name, 'x')
  t.is(node.pattern, null)
})

// ── spread expression ─────────────────────────────────────────────

test('lower JS spread in call arguments', (t) => {
  const node = lower('fn(...args);', 'javascript').body[0]
  if (node.type !== 'expressionStatement') return t.fail()
  if (node.expression.type !== 'call') return t.fail()
  t.is(node.expression.arguments[0].type, 'spreadExpr')
  if (node.expression.arguments[0].type !== 'spreadExpr') return t.fail()
  t.is(node.expression.arguments[0].argument.type, 'identifier')
})

test('lower JS spread in array literal', (t) => {
  const node = lower('[...arr, 1];', 'javascript').body[0]
  if (node.type !== 'expressionStatement') return t.fail()
  if (node.expression.type !== 'arrayExpr') return t.fail()
  const first = node.expression.elements[0]
  if (!first) return t.fail()
  t.is(first.type, 'spreadExpr')
})
