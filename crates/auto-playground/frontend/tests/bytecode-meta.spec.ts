import { test, expect } from '@playwright/test';

const code = `fn add(a int, b int) int {
  a + b
}
let msg = "greetings from auto"
print(msg)
let r = add(3, 4)
print(r)`;

test.describe('Bytecode panel meta tooltips', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForSelector('.cm-content', { timeout: 10000 });
    await page.click('.cm-content');
    await page.keyboard.press('Control+a');
    await page.keyboard.type(code);
    await page.click('.run-btn');
    await page.waitForSelector('.bytecode-line', { timeout: 15000 });
  });

  test('bytecode panel stretches to the right window edge', async ({ page }) => {
    const gap = await page.evaluate(() => {
      const panel = document.querySelector('.bytecode-panel')!.getBoundingClientRect();
      return window.innerWidth - panel.right;
    });
    expect(gap).toBeLessThanOrEqual(2);
  });

  test('operand references resolve to concrete values on hover', async ({ page }) => {
    // str[0] resolves to the actual string constant
    const strTok = page.locator('.tok', { hasText: 'str[0]' }).first();
    await expect(strTok).toHaveAttribute('data-tip', /greetings from auto/);
    await strTok.hover();
    await expect(page.locator('.tok-tooltip')).toContainText('greetings from auto');

    // nat#N resolves to the native shim name
    await expect(
      page.locator('.tok', { hasText: 'nat#' }).first()
    ).toHaveAttribute('data-tip', /auto\.print/);

    // call target 0x0008 resolves to fn add
    await expect(page.locator('.tok', { hasText: '0x0008' }).first())
      .toHaveAttribute('data-tip', 'fn add');
  });

  test('tooltip hides when leaving the panel', async ({ page }) => {
    const strTok = page.locator('.tok', { hasText: 'str[0]' }).first();
    await strTok.hover();
    await expect(page.locator('.tok-tooltip')).toBeVisible();
    await page.mouse.move(200, 300);
    await expect(page.locator('.tok-tooltip')).toBeHidden();
  });
});
