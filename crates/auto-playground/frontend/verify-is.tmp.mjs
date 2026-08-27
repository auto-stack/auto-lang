import { chromium } from '@playwright/test';
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1600, height: 900 } });
await page.goto('http://127.0.0.1:3030/');
await page.waitForSelector('.cm-content', { timeout: 10000 });
await page.selectOption('select.example-selector', { label: 'Pattern-Matching' });
await page.waitForTimeout(300);
await page.click('.run-btn');
await page.waitForSelector('.bytecode-line', { timeout: 15000 });
await page.waitForTimeout(300);

// Click source line 22 (first is-arm) — must highlight bytecode
const g22 = page.locator('.cm-lineNumbers .cm-gutterElement').filter({ hasText: /^22$/ });
await g22.click();
await page.waitForTimeout(250);
const arm1 = await page.evaluate(() => [...document.querySelectorAll('.bytecode-line.is-highlighted')].map(e => e.dataset.offset));
console.log('line 22 -> offsets:', JSON.stringify(arm1));

// Line 23 (second arm) — different set
await page.locator('.cm-lineNumbers .cm-gutterElement').filter({ hasText: /^23$/ }).click();
await page.waitForTimeout(250);
const arm2 = await page.evaluate(() => [...document.querySelectorAll('.bytecode-line.is-highlighted')].map(e => e.dataset.offset));
console.log('line 23 -> offsets:', JSON.stringify(arm2));

if (arm1.length === 0) throw new Error('arm 1 (line 22) has no bytecode mapping');
if (arm2.length === 0) throw new Error('arm 2 (line 23) has no bytecode mapping');
const overlap = arm1.filter(o => arm2.includes(o));
if (overlap.length) throw new Error('arm highlights overlap: ' + JSON.stringify(overlap));

// Lines 10-12 (is c arms) too
await page.locator('.cm-lineNumbers .cm-gutterElement').filter({ hasText: /^10$/ }).click();
await page.waitForTimeout(250);
const arm10 = await page.evaluate(() => document.querySelectorAll('.bytecode-line.is-highlighted').length);
console.log('line 10 highlighted count:', arm10);
if (arm10 === 0) throw new Error('first is-block arm has no mapping');

console.log('IS-ARM LINKING VERIFICATION PASSED');
await browser.close();
