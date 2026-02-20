import test from 'ava'

import type { CstNode } from '../index'
import { parse } from '../index'

test('parse JS variable declaration', (t) => {
  const cst = parse('let x = 1;', 'javascript')
  t.is(cst.kind, 'program')
  t.true(cst.children.some((c) => c.kind === 'lexical_declaration'))
})

test('parse TS variable declaration with type annotation', (t) => {
  const cst = parse('let x: number = 1;', 'typescript')
  t.is(cst.kind, 'program')
  const decl = cst.children.find((c) => c.kind === 'lexical_declaration')
  t.truthy(decl)
})

test('parse TSX element', (t) => {
  const cst = parse('const el = <div />;', 'tsx')
  t.is(cst.kind, 'program')
})

test('parse accepts language shorthand', (t) => {
  const cst = parse('let x = 1;', 'js')
  t.is(cst.kind, 'program')
})

test('leaf nodes have text, branch nodes do not', (t) => {
  const cst = parse('let x = 1;', 'js')
  t.is(cst.text, null)

  const findLeaf = (node: CstNode): CstNode | null => {
    if (node.kind === 'identifier' && node.text) return node
    for (const child of node.children) {
      const found = findLeaf(child)
      if (found) return found
    }
    return null
  }
  const ident = findLeaf(cst)
  if (!ident) return t.fail()
  t.is(ident.text, 'x')
})

test('CST nodes have span with line, column, offset', (t) => {
  const cst = parse('let x = 1;', 'js')
  t.is(cst.span.start.line, 0)
  t.is(cst.span.start.column, 0)
  t.is(cst.span.start.offset, 0)
})

test('parse throws on unsupported language', (t) => {
  t.throws(() => parse('let x = 1;', 'python'), {
    message: /unsupported language/,
  })
})
