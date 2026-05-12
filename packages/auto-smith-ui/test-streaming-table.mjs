import { chromium } from 'playwright'

const URL = 'http://localhost:5181/forge/'

async function main() {
  const browser = await chromium.launch({ headless: true })
  const page = await browser.newPage()
  await page.goto(URL)
  await page.waitForTimeout(1000)

  // Click the Demo tab
  const demoTab = await page.locator('.rail-tab', { hasText: '演示' })
  await demoTab.click()
  await page.waitForTimeout(500)

  // Poll DOM at 30ms intervals (finer than MutationObserver)
  const samples = await page.evaluate(() => {
    return new Promise((resolve) => {
      const samples = []
      const start = performance.now()
      const id = setInterval(() => {
        const doc = document.querySelector('.streaming-document')
        const tables = document.querySelectorAll('.streaming-table')
        const loadingRows = document.querySelectorAll('.loading-row')
        const dataRows = document.querySelectorAll('.streaming-table tbody tr:not(.loading-row)')
        const ths = document.querySelectorAll('.streaming-table th')

        samples.push({
          t: Math.round(performance.now() - start),
          hasTable: tables.length > 0,
          hasLoading: loadingRows.length > 0,
          dataRows: dataRows.length,
          cols: ths.length,
          htmlLen: doc ? doc.innerHTML.length : 0,
        })
      }, 30)

      // Click Data Table after a short delay
      setTimeout(() => {
        document.querySelector('button.demo-btn:nth-child(2)')?.click()
      }, 100)

      // Stop after 10 seconds
      setTimeout(() => {
        clearInterval(id)
        resolve(samples)
      }, 10000)
    })
  })

  console.log('=== Fine-grained DOM Poll (30ms) ===')
  let prevHasTable = false
  let prevHasLoading = false
  let tableAppearances = 0
  let loadingAppearances = 0

  for (const s of samples) {
    if (s.hasTable && !prevHasTable) tableAppearances++
    if (s.hasLoading && !prevHasLoading) loadingAppearances++
    prevHasTable = s.hasTable
    prevHasLoading = s.hasLoading

    // Print only state-change rows
    const prev = samples[samples.indexOf(s) - 1]
    const changed = !prev ||
      prev.hasTable !== s.hasTable ||
      prev.hasLoading !== s.hasLoading ||
      prev.dataRows !== s.dataRows ||
      prev.cols !== s.cols

    if (changed) {
      console.log(`t=${String(s.t).padStart(4)}ms  table=${s.hasTable}  loading=${s.hasLoading}  dataRows=${s.dataRows}  cols=${s.cols}  htmlLen=${s.htmlLen}`)
    }
  }

  console.log('\n=== Summary ===')
  console.log('Table appearances:', tableAppearances)
  console.log('Loading appearances:', loadingAppearances)
  console.log('Total samples:', samples.length)

  await page.screenshot({ path: 'test-streaming-table-final.png', fullPage: false })
  await browser.close()
}

main().catch((e) => {
  console.error(e)
  process.exit(1)
})
