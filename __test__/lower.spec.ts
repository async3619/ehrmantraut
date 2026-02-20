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
