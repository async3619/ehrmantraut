import test from 'ava'

import type { SourceEntry } from '../index'
import { parseBatch, lowerBatch } from '../index'

// ── parseBatch ────────────────────────────────────────────────────

test('parseBatch processes multiple entries', async (t) => {
  const entries: SourceEntry[] = [
    { source: 'let x = 1;', language: 'javascript' },
    { source: 'let y: number = 2;', language: 'typescript' },
    { source: 'const el = <div />;', language: 'tsx' },
  ]
  const results = await parseBatch(entries)
  t.is(results.length, 3)
  for (const r of results) {
    t.true(r.success)
    if (r.success) t.is(r.result.kind, 'program')
  }
})

test('parseBatch captures per-entry errors without aborting', async (t) => {
  const entries: SourceEntry[] = [
    { source: 'let x = 1;', language: 'javascript' },
    { source: 'let y = 2;', language: 'python' },
    { source: 'let z = 3;', language: 'typescript' },
  ]
  const results = await parseBatch(entries)
  t.is(results.length, 3)
  t.true(results[0].success)
  t.false(results[1].success)
  if (!results[1].success) t.regex(results[1].error, /unsupported language/)
  t.true(results[2].success)
})

test('parseBatch handles empty array', async (t) => {
  const results = await parseBatch([])
  t.is(results.length, 0)
})

test('parseBatch preserves entry order', async (t) => {
  const entries: SourceEntry[] = Array.from({ length: 20 }, (_, i) => ({
    source: `let x${i} = ${i};`,
    language: 'javascript' as const,
  }))
  const results = await parseBatch(entries)
  t.is(results.length, 20)
  for (const r of results) {
    t.true(r.success)
  }
})

// ── lowerBatch ────────────────────────────────────────────────────

test('lowerBatch processes multiple entries', async (t) => {
  const entries: SourceEntry[] = [
    { source: 'let x = 1;', language: 'javascript' },
    { source: 'const y = 2;', language: 'typescript' },
  ]
  const results = await lowerBatch(entries)
  t.is(results.length, 2)
  for (const r of results) {
    t.true(r.success)
    if (r.success) t.truthy(r.result.body)
  }
})

test('lowerBatch captures per-entry errors', async (t) => {
  const entries: SourceEntry[] = [
    { source: 'let x = 1;', language: 'js' },
    { source: 'let y = 2;', language: 'python' },
  ]
  const results = await lowerBatch(entries)
  t.true(results[0].success)
  t.false(results[1].success)
})

test('lowerBatch handles empty array', async (t) => {
  const results = await lowerBatch([])
  t.is(results.length, 0)
})
