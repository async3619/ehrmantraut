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
