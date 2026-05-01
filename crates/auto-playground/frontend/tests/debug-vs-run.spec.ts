import { test, expect } from '@playwright/test';

test.describe('Debug mode result matches Run mode', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    // Wait for the editor to be ready
    await page.waitForSelector('.cm-content', { timeout: 10000 });
  });

  test('expression result is the same in Run and Debug', async ({ page }) => {
    const source = '1 + 2';

    // Clear editor and type source
    await page.click('.cm-content');
    await page.keyboard.press('Control+a');
    await page.keyboard.type(source);

    // Click Run and wait for result
    await page.click('.run-btn');
    await page.waitForFunction(() => {
      const el = document.querySelector('.console-main');
      return el && el.textContent?.includes('Result: 3');
    }, { timeout: 15000 });

    const runResult = await page.locator('.console-main').textContent();
    expect(runResult).toContain('Result: 3');

    // Now click Debug
    await page.click('.debug-btn');

    // Wait for debug to connect and pause
    await page.waitForSelector('.debug-btn.active', { timeout: 10000 });
    await page.waitForFunction(() => {
      const el = document.querySelector('.debug-aux-panel');
      return el && el.textContent?.includes('No variables');
    }, { timeout: 10000 });

    // Press F5 (Continue) to run to completion
    await page.keyboard.press('F5');

    // Wait for debug to finish
    await page.waitForFunction(() => {
      const btn = document.querySelector('.debug-btn');
      return btn && !btn.classList.contains('active');
    }, { timeout: 15000 });

    // After debug finishes, the main console should show the same result
    const debugResult = await page.locator('.console-main').textContent();
    expect(debugResult).toContain('Result: 3');
  });

  test('stdout is the same in Run and Debug', async ({ page }) => {
    const source = 'print("hello")';

    // Clear editor and type source
    await page.click('.cm-content');
    await page.keyboard.press('Control+a');
    await page.keyboard.type(source);

    // Click Run and wait for output
    await page.click('.run-btn');
    await page.waitForFunction(() => {
      const el = document.querySelector('.console-main');
      return el && el.textContent?.includes('hello');
    }, { timeout: 15000 });

    const runOutput = await page.locator('.console-main').textContent();
    expect(runOutput).toContain('hello');

    // Now click Debug
    await page.click('.debug-btn');
    await page.waitForSelector('.debug-btn.active', { timeout: 10000 });

    // Press F5 to continue
    await page.keyboard.press('F5');

    // Wait for debug to finish
    await page.waitForFunction(() => {
      const btn = document.querySelector('.debug-btn');
      return btn && !btn.classList.contains('active');
    }, { timeout: 15000 });

    // Main console should show the same stdout
    const debugOutput = await page.locator('.console-main').textContent();
    expect(debugOutput).toContain('hello');
  });
});
