import { test, expect } from '@playwright/test';

test.describe('Debug mode toolbar and shortcuts', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForSelector('.cm-content', { timeout: 10000 });
    await page.click('.cm-content');
    await page.keyboard.press('Control+a');
    await page.keyboard.type('1 + 2');
  });

  test('Run and Trans are hidden while debugging, Debug button becomes Exit', async ({ page }) => {
    await page.click('.debug-btn');
    await page.waitForSelector('.debug-btn.active', { timeout: 10000 });

    // Run/Trans hidden in debug mode
    await expect(page.locator('.run-btn')).toBeHidden();
    await expect(page.locator('.trans-split-btn')).toBeHidden();

    // Debug button turns into Exit Debug
    const debugBtn = page.locator('.debug-btn');
    await expect(debugBtn).toHaveText(/Exit Debug/);

    // Exit via the button returns to normal toolbar
    await debugBtn.click();
    await page.waitForFunction(() => {
      const btn = document.querySelector('.debug-btn');
      return btn && !btn.classList.contains('active');
    }, { timeout: 10000 });
    await expect(page.locator('.run-btn')).toBeVisible();
    await expect(page.locator('.trans-split-btn')).toBeVisible();
    await expect(debugBtn).toHaveText(/Debug/);
  });

  test('Ctrl+Enter does not run while debugging, Shift+F5 exits debug', async ({ page }) => {
    await page.click('.debug-btn');
    await page.waitForSelector('.debug-btn.active', { timeout: 10000 });

    // Ctrl+Enter must not trigger a Run while debugging (mode stays debug,
    // no run output pane swap)
    await page.keyboard.press('Control+Enter');
    await page.waitForTimeout(500);
    await expect(page.locator('.debug-btn')).toHaveClass(/active/);

    // Shift+F5 stops the debug session
    await page.keyboard.press('Shift+F5');
    await page.waitForFunction(() => {
      const btn = document.querySelector('.debug-btn');
      return btn && !btn.classList.contains('active');
    }, { timeout: 10000 });
    await expect(page.locator('.run-btn')).toBeVisible();
  });
});
