import { execSync } from 'node:child_process'
import fs from 'node:fs'
import path from 'node:path'

import fg from 'fast-glob'

import type { SourceEntry as BatchSourceEntry } from '../index.js'
import { lower, lowerAsync, lowerBatch, parse, parseAsync, parseBatch } from '../index.js'

const BENCH_DIR = '/tmp/ehrmantraut-bench'

const TARGETS = [
  { name: 'zod', repo: 'https://github.com/colinhacks/zod.git' },
  { name: 'lodash', repo: 'https://github.com/lodash/lodash.git' },
  { name: 'react', repo: 'https://github.com/facebook/react.git' },
  { name: 'express', repo: 'https://github.com/expressjs/express.git' },
]

function cloneRepos() {
  fs.mkdirSync(BENCH_DIR, { recursive: true })

  for (const target of TARGETS) {
    const targetDir = path.join(BENCH_DIR, target.name)
    if (fs.existsSync(targetDir)) {
      console.log(`  [skip] ${target.name} (already cloned)`)
      continue
    }
    console.log(`  [clone] ${target.name}...`)
    execSync(`git clone --depth 1 ${target.repo} ${targetDir}`, { stdio: 'pipe' })
  }
}

function collectFiles(dir: string): string[] {
  return fg.sync('**/*.{js,ts,tsx,mjs,mts}', {
    cwd: dir,
    absolute: true,
    ignore: ['**/node_modules/**', '**/.git/**', '**/dist/**', '**/build/**', '**/__snapshots__/**', '**/coverage/**'],
  })
}

function detectLanguage(filePath: string): string {
  const ext = path.extname(filePath)
  if (ext === '.tsx') return 'tsx'
  if (ext === '.ts' || ext === '.mts') return 'typescript'
  return 'javascript'
}

interface SourceEntry {
  source: string
  lang: string
}

interface LibrarySources {
  name: string
  sources: SourceEntry[]
  files: number
  lines: number
  bytes: number
}

interface BenchResult {
  library: string
  phase: string
  mode: string
  files: number
  lines: number
  bytes: number
  timeMs: number
  errors: number
  throughputLinesPerSec: string
  throughputMbPerSec: string
}

function loadLibrary(name: string): LibrarySources {
  const dir = path.join(BENCH_DIR, name)
  const files = collectFiles(dir)

  let lines = 0
  let bytes = 0
  const sources: SourceEntry[] = []

  for (const file of files) {
    const source = fs.readFileSync(file, 'utf-8')
    lines += source.split('\n').length
    bytes += Buffer.byteLength(source, 'utf-8')
    sources.push({ source, lang: detectLanguage(file) })
  }

  return { name, sources, files: files.length, lines, bytes }
}

function toBenchResult(
  lib: LibrarySources,
  phase: string,
  mode: string,
  elapsed: number,
  errors: number,
): BenchResult {
  const throughputLinesPerSec = ((lib.lines / elapsed) * 1000).toFixed(0)
  const throughputMbPerSec = ((lib.bytes / 1024 / 1024 / elapsed) * 1000).toFixed(2)

  return {
    library: lib.name,
    phase,
    mode,
    files: lib.files,
    lines: lib.lines,
    bytes: lib.bytes,
    timeMs: Math.round(elapsed),
    errors,
    throughputLinesPerSec: `${Number(throughputLinesPerSec).toLocaleString()} lines/s`,
    throughputMbPerSec: `${throughputMbPerSec} MB/s`,
  }
}

// ── Sync benchmark ────────────────────────────────────────────────

function runSync(lib: LibrarySources, phase: 'parse' | 'lower'): BenchResult {
  const fn = phase === 'parse' ? parse : lower
  let errors = 0

  const start = performance.now()
  for (const { source, lang } of lib.sources) {
    try {
      fn(source, lang)
    } catch {
      errors++
    }
  }
  const elapsed = performance.now() - start

  return toBenchResult(lib, phase, 'sync', elapsed, errors)
}

// ── Async benchmark (sequential awaits) ───────────────────────────

async function runAsync(lib: LibrarySources, phase: 'parse' | 'lower'): Promise<BenchResult> {
  const fn = phase === 'parse' ? parseAsync : lowerAsync
  let errors = 0

  const start = performance.now()
  for (const { source, lang } of lib.sources) {
    try {
      await fn(source, lang)
    } catch {
      errors++
    }
  }
  const elapsed = performance.now() - start

  return toBenchResult(lib, phase, 'async', elapsed, errors)
}

// ── Batch benchmark (rayon parallel) ──────────────────────────────

async function runBatch(lib: LibrarySources, phase: 'parse' | 'lower'): Promise<BenchResult> {
  const fn = phase === 'parse' ? parseBatch : lowerBatch
  const entries: BatchSourceEntry[] = lib.sources.map((s) => ({ source: s.source, language: s.lang }))

  const start = performance.now()
  const results = await fn(entries)
  const elapsed = performance.now() - start

  const errors = results.filter((r) => !r.success).length

  return toBenchResult(lib, phase, 'batch', elapsed, errors)
}

// ── Output ────────────────────────────────────────────────────────

function printResults(title: string, results: BenchResult[]) {
  console.log(`\n${title}\n`)
  console.table(
    results.map((r) => ({
      Library: r.library,
      Mode: r.mode,
      Files: r.files,
      Lines: r.lines.toLocaleString(),
      'Size (KB)': Math.round(r.bytes / 1024).toLocaleString(),
      'Time (ms)': r.timeMs,
      Errors: r.errors,
      Throughput: r.throughputLinesPerSec,
      'MB/s': r.throughputMbPerSec,
    })),
  )
}

function printSummary(label: string, results: BenchResult[]) {
  const byMode = new Map<string, BenchResult[]>()
  for (const r of results) {
    const list = byMode.get(r.mode) ?? []
    list.push(r)
    byMode.set(r.mode, list)
  }

  for (const [mode, modeResults] of byMode) {
    const totalFiles = modeResults.reduce((s, r) => s + r.files, 0)
    const totalLines = modeResults.reduce((s, r) => s + r.lines, 0)
    const totalBytes = modeResults.reduce((s, r) => s + r.bytes, 0)
    const totalTime = modeResults.reduce((s, r) => s + r.timeMs, 0)
    const totalErrors = modeResults.reduce((s, r) => s + r.errors, 0)

    console.log(
      `  ${label} [${mode}]: ${totalFiles} files, ${totalLines.toLocaleString()} lines, ${Math.round(totalBytes / 1024).toLocaleString()} KB`,
    )
    console.log(
      `  Time: ${totalTime}ms | Throughput: ${((totalLines / totalTime) * 1000).toFixed(0).replace(/\B(?=(\d{3})+(?!\d))/g, ',')} lines/s | Errors: ${totalErrors}`,
    )
  }
}

// --- Main ---

async function main() {
  console.log('\n📦 Preparing repositories...\n')
  cloneRepos()

  console.log('\n🔬 Running benchmarks...\n')

  const parseResults: BenchResult[] = []
  const lowerResults: BenchResult[] = []

  for (const target of TARGETS) {
    process.stdout.write(`  [load] ${target.name}...`)
    const lib = loadLibrary(target.name)
    console.log(` ${lib.files} files, ${lib.lines.toLocaleString()} lines`)

    // Parse benchmarks
    process.stdout.write(`    parse (sync)...`)
    const parseSyncResult = runSync(lib, 'parse')
    console.log(` ${parseSyncResult.timeMs}ms`)
    parseResults.push(parseSyncResult)

    process.stdout.write(`    parse (async)...`)
    const parseAsyncResult = await runAsync(lib, 'parse')
    console.log(` ${parseAsyncResult.timeMs}ms`)
    parseResults.push(parseAsyncResult)

    process.stdout.write(`    parse (batch)...`)
    const parseBatchResult = await runBatch(lib, 'parse')
    console.log(` ${parseBatchResult.timeMs}ms`)
    parseResults.push(parseBatchResult)

    // Lower benchmarks
    process.stdout.write(`    lower (sync)...`)
    const lowerSyncResult = runSync(lib, 'lower')
    console.log(` ${lowerSyncResult.timeMs}ms`)
    lowerResults.push(lowerSyncResult)

    process.stdout.write(`    lower (async)...`)
    const lowerAsyncResult = await runAsync(lib, 'lower')
    console.log(` ${lowerAsyncResult.timeMs}ms`)
    lowerResults.push(lowerAsyncResult)

    process.stdout.write(`    lower (batch)...`)
    const lowerBatchResult = await runBatch(lib, 'lower')
    console.log(` ${lowerBatchResult.timeMs}ms`)
    lowerResults.push(lowerBatchResult)
  }

  printResults('📊 Parse Results:', parseResults)
  printResults('📊 Lower Results:', lowerResults)

  console.log('\n📈 Summary:\n')
  printSummary('Parse', parseResults)
  printSummary('Lower', lowerResults)
}

main()
