import test from 'ava'

import { lower } from '../index'

test('lower JS let declaration', (t) => {
  const ir = lower('let x = 1;', 'javascript')
  t.truthy(ir.body)
  t.is(ir.body.length, 1)
  t.is(ir.body[0].type, 'variableDecl')
  t.is(ir.body[0].name, 'x')
  t.is(ir.body[0].annotations.declarationKind, 'let')
  t.is(ir.body[0].annotations.scopeLevel, 'block')
})

test('lower JS var declaration has function scope level', (t) => {
  const ir = lower('var x = 1;', 'javascript')
  t.is(ir.body[0].annotations.declarationKind, 'var')
  t.is(ir.body[0].annotations.scopeLevel, 'function')
})

test('lower JS const declaration', (t) => {
  const ir = lower('const x = 1;', 'javascript')
  t.is(ir.body[0].annotations.declarationKind, 'const')
  t.is(ir.body[0].annotations.scopeLevel, 'block')
})

test('lower JS function declaration', (t) => {
  const ir = lower('function foo(a, b) { return a + b; }', 'javascript')
  t.is(ir.body[0].type, 'functionDecl')
  t.is(ir.body[0].name, 'foo')
  t.is(ir.body[0].params.length, 2)
})

test('lower JS if statement', (t) => {
  const ir = lower('if (x) { y; }', 'javascript')
  t.is(ir.body[0].type, 'if')
  t.truthy(ir.body[0].condition)
  t.truthy(ir.body[0].consequent)
})

test('lower JS for statement', (t) => {
  const ir = lower('for (let i = 0; i < 10; i++) { x; }', 'javascript')
  t.is(ir.body[0].type, 'for')
  t.truthy(ir.body[0].body)
})

test('lower JS while statement', (t) => {
  const ir = lower('while (true) { x; }', 'javascript')
  t.is(ir.body[0].type, 'while')
  t.is(ir.body[0].isDoWhile, false)
})

test('lower JS try-catch statement', (t) => {
  const ir = lower('try { x; } catch (e) { y; } finally { z; }', 'javascript')
  t.is(ir.body[0].type, 'tryCatch')
  t.truthy(ir.body[0].tryBlock)
  t.truthy(ir.body[0].catchClause)
  t.truthy(ir.body[0].finallyBlock)
})

test('lower JS expression statement with call', (t) => {
  const ir = lower('console.log("hello");', 'javascript')
  t.is(ir.body[0].type, 'expressionStatement')
  t.is(ir.body[0].expression.type, 'call')
})

test('lower TS ignores type annotations', (t) => {
  const ir = lower('let x: number = 1;', 'typescript')
  t.is(ir.body[0].type, 'variableDecl')
  t.is(ir.body[0].name, 'x')
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
  t.is(ir.body[0].name, 'x')
  t.is(ir.body[1].name, 'y')
})
