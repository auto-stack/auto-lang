import { test, expect } from '@playwright/test'

test.describe('TAPL example runners', () => {
  test('Listing Run sends edited source and the bundled file in EN and ZH', async ({ page }) => {
    const requests: Array<Record<string, unknown>> = []
    await page.route('**/api/run', async (route) => {
      requests.push(route.request().postDataJSON() as Record<string, unknown>)
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ stdout: 'Edited source ran\n', stderr: '', exit_code: 0 }),
      })
    })

    for (const [path, locale, label] of [
      ['/books/tapl/ch14-file-processor', 'en', 'Run'],
      ['/zh/books/tapl/ch14-file-processor', 'zh', '运行'],
    ] as const) {
      await page.goto(path, { waitUntil: 'domcontentloaded' })
      const id = `${locale}/ch14-file-processor/listing-14-2`
      const view = page.locator(`.code-view[data-example-id="${id}"]`)
      await expect(view).toBeVisible()
      await expect(view.locator('.run-btn')).toHaveText(new RegExp(label))
      await view.locator('.cm-editor .cm-content').fill('fn main() {\n    print("Edited source ran")\n}')
      await view.locator('.run-btn').click()
      await expect(view.locator('.output-content')).toHaveText('Edited source ran')
    }

    expect(requests).toHaveLength(2)
    for (const request of requests) {
      expect(request.entry).toBe('main.at')
      expect(request.source).toContain('Edited source ran')
      expect(request.input).toBe('')
      expect(request.timeout_secs).toBe(10)
      expect(request.files).toEqual([{ path: 'input.txt', source: 'Hello from file!' }])
    }
  })

  test('editable input is included in a Run request', async ({ page }) => {
    let requestBody: Record<string, unknown> | undefined
    await page.route('**/api/run', async (route) => {
      requestBody = route.request().postDataJSON() as Record<string, unknown>
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ stdout: 'Input received\n', stderr: '', exit_code: 0 }),
      })
    })
    await page.goto('/books/tapl/ch05-guessing-game', { waitUntil: 'domcontentloaded' })
    const view = page.locator('.code-view[data-example-id="en/ch05-guessing-game/listing-5-4"]')
    await expect(view).toBeVisible()
    await expect(view.locator('.cm-editor')).toBeVisible()
    await view.locator('.input-pane textarea').fill('17\n')
    const response = page.waitForResponse((res) => res.url().endsWith('/api/run') && res.request().method() === 'POST')
    await view.locator('.run-btn').click()
    await response
    await expect(view.locator('.output-content')).toHaveText('Input received')
    expect(requestBody?.input).toBe('17\n')
  })

  test('test listings call /api/test and render assertion results', async ({ page }) => {
    let requestedUrl = ''
    let requestBody: Record<string, unknown> | undefined
    await page.route('**/api/test', async (route) => {
      requestedUrl = route.request().url()
      requestBody = route.request().postDataJSON() as Record<string, unknown>
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ executed: 1, passed: 1, failed: 0, reports: [] }),
      })
    })
    await page.goto('/books/tapl/ch18-testing', { waitUntil: 'domcontentloaded' })
    const view = page.locator('.code-view[data-example-id="en/ch18-testing/listing-18-1"]')
    await expect(view).toBeVisible()
    await expect(view.locator('.cm-editor')).toBeVisible()
    const response = page.waitForResponse((res) => res.url().endsWith('/api/test') && res.request().method() === 'POST')
    await view.locator('.run-btn').click()
    await response
    await expect(view.locator('.output-content')).toHaveText('1/1 tests passed')
    expect(requestedUrl).toContain('/api/test')
    expect(requestBody?.source).toContain('#[test]')
  })

  test('AutoFence runner executes the edited full source', async ({ page }) => {
    let requestBody: Record<string, unknown> | undefined
    await page.route('**/api/run', async (route) => {
      requestBody = route.request().postDataJSON() as Record<string, unknown>
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ stdout: 'Fence source ran\n', stderr: '', exit_code: 0 }),
      })
    })
    await page.goto('/books/tapl/ch01-getting-started', { waitUntil: 'domcontentloaded' })
    const fence = page.locator('.auto-fence[data-example-id="en/ch01-getting-started/fence-014"]')
    await fence.locator('.af-run').click()
    const runner = fence.locator('.tapl-runner')
    await expect(runner).toBeVisible()
    await runner.locator('textarea.source').fill('fn main() {\n    print("Fence source ran")\n}')
    await runner.locator('button.run').click()
    await expect(runner.locator('.result pre')).toHaveText('Fence source ran')
    expect(requestBody?.source).toContain('Fence source ran')
    expect(requestBody?.entry).toBe('main.at')
  })

  test('display-only TOML listing has no Run action', async ({ page }) => {
    await page.goto('/books/tapl/ch01-getting-started', { waitUntil: 'domcontentloaded' })
    const listing = page.locator('.code-view[data-example-id="en/ch01-getting-started/listing-1-2"]')
    await expect(listing).toBeVisible()
    await expect(listing.locator('.run-btn')).toHaveCount(0)
  })
})
