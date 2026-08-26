import { test, expect } from '@playwright/test';

const code = 'var a = [5, 1, 4]\n\na.sort()\n\nprint(a[0])';

test.describe('Run-mode source ↔ bytecode linking', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForSelector('.cm-content', { timeout: 10000 });
    await page.click('.cm-content');
    await page.keyboard.press('Control+a');
    await page.keyboard.type(code);
    await page.click('.run-btn');
    await page.waitForSelector('.bytecode-line', { timeout: 15000 });
  });

  test('bytecode carries source line info for top-level statements', async ({ page }) => {
    const withSource = await page.locator('.bytecode-line.has-source').count();
    expect(withSource).toBeGreaterThan(0);
  });

  test('clicking a source line highlights its bytecode', async ({ page }) => {
    await page.locator('.cm-lineNumbers .cm-gutterElement').filter({ hasText: /^3$/ }).click();
    const highlighted = page.locator('.bytecode-line.is-highlighted');
    await expect(highlighted.first()).toBeVisible();
    const offsets3 = await highlighted.evaluateAll(els =>
      els.map(el => (el as HTMLElement).dataset.offset));

    await page.locator('.cm-lineNumbers .cm-gutterElement').filter({ hasText: /^5$/ }).click();
    const offsets5 = await highlighted.evaluateAll(els =>
      els.map(el => (el as HTMLElement).dataset.offset));

    expect(offsets3.length).toBeGreaterThan(0);
    expect(offsets5.length).toBeGreaterThan(0);
    expect(new Set(offsets3).size === offsets3.length).toBe(true);
    expect(offsets5).not.toEqual(offsets3);
  });

  test('clicking a bytecode line highlights its source line', async ({ page }) => {
    await page.locator('.bytecode-line.has-source').nth(3).click();
    const highlighted = await page.evaluate(() => {
      const lines = [...document.querySelectorAll('.cm-content .cm-line')];
      return lines.findIndex(el => {
        const bg = getComputedStyle(el).backgroundColor;
        return bg !== 'rgba(0, 0, 0, 0)' && bg !== 'transparent';
      });
    });
    expect(highlighted).toBeGreaterThanOrEqual(0);
  });
});
