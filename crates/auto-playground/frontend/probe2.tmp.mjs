import { chromium } from '@playwright/test';
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1600, height: 720 } });
await page.goto('http://127.0.0.1:3030/');
await page.waitForSelector('.cm-content', { timeout: 10000 });
await page.selectOption('select.example-selector', { label: 'Pattern-Matching' });
await page.waitForTimeout(300);
await page.click('.run-btn');
await page.waitForSelector('.bytecode-line', { timeout: 15000 });
await page.waitForTimeout(300);
await page.locator('.cm-lineNumbers .cm-gutterElement').filter({ hasText: /^22$/ }).click();
await page.waitForTimeout(300);
const state = await page.evaluate(() => ({
  bcSelected: document.querySelectorAll('.bytecode-line.is-selected').length,
  cmSelected: document.querySelectorAll('.cm-content .cm-selected-line').length,
  cmCross: document.querySelectorAll('.cm-content .cm-cross-highlight-line').length,
}));
console.log(JSON.stringify(state));
// shift check regardless
const shift = await page.evaluate(() => {
  const lines = [...document.querySelectorAll('.cm-content .cm-line')];
  const sel = lines.findIndex(el => el.classList.contains('cm-selected-line'));
  if (sel < 1) return { sel };
  return { sel, deltaX: Math.round((lines[sel].getBoundingClientRect().left - lines[sel-1].getBoundingClientRect().left) * 10) / 10 };
});
console.log('shift:', JSON.stringify(shift));
await browser.close();
