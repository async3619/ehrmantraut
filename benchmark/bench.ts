import { execSync } from 'node:child_process'
import fs from 'node:fs'
import path from 'node:path'

import fg from 'fast-glob'

import { lower, parse } from '../index.js'

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

function runBenchmark(lib: LibrarySources, phase: 'parse' | 'lower'): BenchResult {
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

  const throughputLinesPerSec = ((lib.lines / elapsed) * 1000).toFixed(0)
  const throughputMbPerSec = ((lib.bytes / 1024 / 1024 / elapsed) * 1000).toFixed(2)

  return {
    library: lib.name,
    phase,
    files: lib.files,
    lines: lib.lines,
    bytes: lib.bytes,
    timeMs: Math.round(elapsed),
    errors,
    throughputLinesPerSec: `${Number(throughputLinesPerSec).toLocaleString()} lines/s`,
    throughputMbPerSec: `${throughputMbPerSec} MB/s`,
  }
}

function printResults(title: string, results: BenchResult[]) {
  console.log(`\n${title}\n`)
  console.table(
    results.map((r) => ({
      Library: r.library,
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
  const totalFiles = results.reduce((s, r) => s + r.files, 0)
  const totalLines = results.reduce((s, r) => s + r.lines, 0)
  const totalBytes = results.reduce((s, r) => s + r.bytes, 0)
  const totalTime = results.reduce((s, r) => s + r.timeMs, 0)
  const totalErrors = results.reduce((s, r) => s + r.errors, 0)

  console.log(`  ${label}: ${totalFiles} files, ${totalLines.toLocaleString()} lines, ${Math.round(totalBytes / 1024).toLocaleString()} KB`)
  console.log(`  Time: ${totalTime}ms | Throughput: ${((totalLines / totalTime) * 1000).toFixed(0).replace(/\B(?=(\d{3})+(?!\d))/g, ',')} lines/s | Errors: ${totalErrors}`)
}

// --- Main ---

console.log('\n📦 Preparing repositories...\n')
cloneRepos()

console.log('\n🔬 Running benchmarks...\n')

const parseResults: BenchResult[] = []
const lowerResults: BenchResult[] = []

for (const target of TARGETS) {
  process.stdout.write(`  [load] ${target.name}...`)
  const lib = loadLibrary(target.name)
  console.log(` ${lib.files} files, ${lib.lines.toLocaleString()} lines`)

  process.stdout.write(`    parse...`)
  const parseResult = runBenchmark(lib, 'parse')
  console.log(` ${parseResult.timeMs}ms (${parseResult.errors} errors)`)
  parseResults.push(parseResult)

  process.stdout.write(`    lower...`)
  const lowerResult = runBenchmark(lib, 'lower')
  console.log(` ${lowerResult.timeMs}ms (${lowerResult.errors} errors)`)
  lowerResults.push(lowerResult)
}

printResults('📊 Parse Results:', parseResults)
printResults('📊 Lower Results:', lowerResults)

console.log('\n📈 Summary:\n')
printSummary('Parse', parseResults)
printSummary('Lower', lowerResults)
