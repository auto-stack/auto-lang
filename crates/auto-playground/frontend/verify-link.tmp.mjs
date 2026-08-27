import { chromium } from '@playwright/test';
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });
await page.goto('http://127.0.0.1:3030/');
await page.waitForSelector('.cm-content', { timeout: 10000 });
await page.selectOption('select.example-selector', { label: 'Sorting' });
await page.waitForTimeout(300);
await page.click('.run-btn');
await page.waitForSelector('.bytecode-line', { timeout: 15000 });
await page.waitForTimeout(300);

// find which source lines have bytecode mapping by probing gutter clicks
async function clickGutter(n) {
  await page.locator('.cm-lineNumbers .cm-gutterElement').nth(n).click();
  await page.waitForTimeout(150);
  return page.evaluate(() => document.querySelectorAll('.bytecode-line.is-highlighted').length);
}
let goodLine = -1;
for (const n of [4, 7, 10]) {  // print / sort_by / let lines
  const c = await clickGutter(n);
  console.log(`gutter[${n}] -> highlighted: ${c}`);
  if (c > 0 && goodLine < 0) goodLine = n;
}
if (goodLine < 0) throw new Error('no line produced bytecode highlight');

// scroll bytecode far away, click again, ensure auto-scroll brings it back
await page.evaluate(() => { document.querySelector('.bytecode-panel .scroll-content').scrollTop = 0; });
await page.waitForTimeout(150);
await clickGutter(goodLine);
await page.waitForTimeout(300);
const fwd = await page.evaluate(() => {
  const hi = [...document.querySelectorAll('.bytecode-line.is-highlighted')];
  const panel = document.querySelector('.bytecode-panel').getBoundingClientRect();
  return {
    count: hi.length,
    offsets: hi.map(el => el.dataset.offset),
    firstTop: hi.length ? Math.round(hi[0].getBoundingClientRect().top) : null,
    panelTop: Math.round(panel.top), panelBottom: Math.round(panel.bottom),
  };
});
console.log('left->right:', JSON.stringify(fwd));
if (fwd.count === 0) throw new Error('no highlight');
if (fwd.firstTop < fwd.panelTop - 2 || fwd.firstTop > fwd.panelBottom + 2) throw new Error('highlight not scrolled into view');

// right -> left: click a has-source bytecode line
await page.locator('.bytecode-line.has-source').nth(20).click();
await page.waitForTimeout(300);
const rev = await page.evaluate(() => {
  const lines = [...document.querySelectorAll('.cm-content .cm-line')];
  const idx = lines.findIndex(el => {
    const bg = getComputedStyle(el).backgroundColor;
    return bg !== 'rgba(0, 0, 0, 0)' && bg !== 'transparent';
  });
  const sc = document.querySelector('.editor-container .cm-scroller');
  return { highlightedIdx: idx, editorScrollTop: Math.round(sc.scrollTop) };
});
console.log('right->left:', JSON.stringify(rev));
if (rev.highlightedIdx < 0) throw new Error('no source highlight after bytecode click');
console.log('LINKING VERIFICATION PASSED');
await browser.close();
