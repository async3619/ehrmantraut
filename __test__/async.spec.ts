import test from 'ava'

import { parse, lower, parseAsync, lowerAsync } from '../index'

// ── parseAsync ────────────────────────────────────────────────────

test('parseAsync returns same result as sync parse', async (t) => {
  const sync = parse('let x = 1;', 'javascript')
  const async_ = await parseAsync('let x = 1;', 'javascript')
  t.deepEqual(async_, sync)
})

test('parseAsync works with typescript', async (t) => {
  const result = await parseAsync('let x: number = 1;', 'typescript')
  t.is(result.kind, 'program')
})

test('parseAsync works with tsx', async (t) => {
  const result = await parseAsync('const el = <div />;', 'tsx')
  t.is(result.kind, 'program')
})

test('parseAsync accepts language shorthand', async (t) => {
  const result = await parseAsync('let x = 1;', 'js')
  t.is(result.kind, 'program')
})

test('parseAsync rejects on unsupported language', async (t) => {
  await t.throwsAsync(() => parseAsync('let x = 1;', 'python'), {
    message: /unsupported language/,
  })
})

// ── lowerAsync ────────────────────────────────────────────────────

test('lowerAsync returns same result as sync lower', async (t) => {
  const sync = lower('let x = 1;', 'javascript')
  const async_ = await lowerAsync('let x = 1;', 'javascript')
  t.deepEqual(async_, sync)
})

test('lowerAsync works with typescript', async (t) => {
  const result = await lowerAsync('let x: number = 1;', 'typescript')
  t.truthy(result.body)
  t.is(result.body.length, 1)
})

test('lowerAsync rejects on unsupported language', async (t) => {
  await t.throwsAsync(() => lowerAsync('let x = 1;', 'python'), {
    message: /unsupported language/,
  })
})
