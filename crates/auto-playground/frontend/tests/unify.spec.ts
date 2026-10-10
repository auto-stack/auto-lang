import { test, expect } from '@playwright/test';

// PLAN-754: ?note=<id> deep link — both standalone and website hosts share
// PlaygroundIDEApp, so the deep-link contract is verified once here.

test.describe('Deep link ?note=', () => {
  test('demo note loads automatically and runs', async ({ page }) => {
    await page.goto('/?note=demo/04-fibonacci');
    await page.waitForSelector('.cm-content', { timeout: 10000 });
    // 编辑器自动载入斐波那契笔记。
    await expect(page.locator('.cm-content')).toContainText('fib', { timeout: 10000 });
    await page.click('.run-btn');
    await page.waitForResponse(
      (res) => res.url().endsWith('/api/run') && res.status() === 200,
      { timeout: 15000 },
    );
    const output = page.locator('.output, [class*="stdout"], [class*="output"]');
    await expect(page.locator('body')).toContainText('0 1 1 2 3 5 8 13 21 34', { timeout: 10000 });
  });

  test('vm-bootstrap note deep link runs with prepend_lib (host-derived)', async ({ page }) => {
    let runPayload: any = null;
    page.on('request', (req) => {
      if (req.url().endsWith('/api/run') && req.method() === 'POST') {
        runPayload = req.postDataJSON();
      }
    });
    await page.goto('/?note=vm-bootstrap/a2r_hello');
    await page.waitForSelector('.cm-content', { timeout: 10000 });
    await expect(page.locator('.cm-content')).toContainText('run_a2r', { timeout: 10000 });
    await page.click('.run-btn');
    await page.waitForResponse(
      (res) => res.url().endsWith('/api/run') && res.status() === 200,
      { timeout: 30000 },
    );
    // 宿主内聚契约：vm-bootstrap 组自动带 prepend_lib。
    expect(runPayload?.prepend_lib).toBe(true);
    await expect(page.locator('body')).toContainText('ok', { timeout: 15000 });
  });

  test('invalid note id falls back to welcome state', async ({ page }) => {
    await page.goto('/?note=demo/no-such-note');
    await page.waitForSelector('.cm-content', { timeout: 10000 });
    // 欢迎态默认源码仍在（不崩、不白屏）。
    await expect(page.locator('.cm-content')).toContainText('add', { timeout: 10000 });
  });
});
