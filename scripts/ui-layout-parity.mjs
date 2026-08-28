// Plan 465 T4 (I6): layout parity runner — asserts the TS port
// (crates/auto-man/assets/wm/layout.ts) against the SAME expectation table
// the Rust engine (auto-lang ui/layout.rs) is held to
// (crates/auto-lang/src/ui/layout_cases.json).
//
// Run: node scripts/ui-layout-parity.mjs
import { readFileSync } from 'node:fs'
import { fileURLToPath, pathToFileURL } from 'node:url'
import path from 'node:path'

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const casesPath = path.join(repoRoot, 'crates', 'auto-lang', 'src', 'ui', 'layout_cases.json')
const table = JSON.parse(readFileSync(casesPath, 'utf-8'))
const wm = await import(
  pathToFileURL(path.join(repoRoot, 'crates', 'auto-man', 'assets', 'wm', 'layout.ts')).href
)

const EPS = 0.51
const [vx, vy, vw, vh] = table.viewport
const viewport = { x: vx, y: vy, width: vw, height: vh }
const reservedBottom = table.reservedTaskbar

function rectMatches(actual, expected) {
  return (
    Math.abs(actual.x - expected[0]) < EPS &&
    Math.abs(actual.y - expected[1]) < EPS &&
    Math.abs(actual.width - expected[2]) < EPS &&
    Math.abs(actual.height - expected[3]) < EPS
  )
}

function assertRect(actual, expected, label) {
  if (!rectMatches(actual, expected)) {
    console.error(
      `FAIL ${label}: got [${actual.x}, ${actual.y}, ${actual.width}, ${actual.height}] want [${expected.join(', ')}]`,
    )
    process.exitCode = 1
    return false
  }
  console.log(`ok   ${label}`)
  return true
}

function makeWins(n, focused, freeRects) {
  return Array.from({ length: n }, (_, i) => ({
    wid: i + 1,
    rect: freeRects ? { x: freeRects[i][0], y: freeRects[i][1], width: freeRects[i][2], height: freeRects[i][3] } : { x: 0, y: 0, width: 1, height: 1 },
    focused: focused === i,
  }))
}

let pass = 0
let fail = 0
for (const c of table.cases) {
  const label = `${c.kind}:${c.name}`
  if (c.kind === 'usable') {
    const u = wm.usableRect(viewport, reservedBottom)
    if (assertRect(u, c.expected, label)) pass++
    else fail++
  } else if (c.kind === 'layout') {
    const wins = makeWins(c.n, c.focused, c.freeRects)
    const rects = wm.layout(c.mode, wins, viewport, reservedBottom)
    const expectedList = c.expectedLast ? [c.expectedLast] : c.expected
    const indices = c.expectedLast ? [rects.length - 1] : rects.map((_, i) => i)
    let caseOk = true
    expectedList.forEach((exp, j) => {
      const r = rects[indices[j]]
      if (!r || !rectMatches(r, exp)) {
        console.error(
          `FAIL ${label}[${indices[j]}]: got ${r ? `[${r.x}, ${r.y}, ${r.width}, ${r.height}]` : 'none'} want [${exp.join(', ')}]`,
        )
        caseOk = false
      }
    })
    if (caseOk) {
      console.log(`ok   ${label}`)
      pass++
    } else fail++
  } else if (c.kind === 'cascade') {
    const usable = wm.usableRect(viewport, reservedBottom)
    const r = wm.cascadeRect(c.index, { width: c.size[0], height: c.size[1] }, usable)
    if (assertRect(r, c.expected, label)) pass++
    else fail++
  } else if (c.kind === 'snap') {
    const usable = wm.usableRect(viewport, reservedBottom)
    const r = wm.snapPreview({ x: c.cursor[0], y: c.cursor[1] }, usable)
    if (c.expected === null) {
      if (r === null) {
        console.log(`ok   ${label}`)
        pass++
      } else {
        console.error(`FAIL ${label}: expected null, got a rect`)
        fail++
      }
    } else if (r && assertRect(r, c.expected, label)) pass++
    else {
      console.error(`FAIL ${label}: expected a rect`)
      fail++
    }
  }
}

console.log(`\nlayout parity: ${pass} pass, ${fail} fail (table: ${path.relative(repoRoot, casesPath)})`)
if (fail > 0) process.exit(1)
