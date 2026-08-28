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
    const selected = page.locator('.bytecode-line.is-selected');
    await expect(selected.first()).toBeVisible();
    const offsets3 = await selected.evaluateAll(els =>
      els.map(el => (el as HTMLElement).dataset.offset));

    await page.locator('.cm-lineNumbers .cm-gutterElement').filter({ hasText: /^5$/ }).click();
    const offsets5 = await selected.evaluateAll(els =>
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

  test('is-statement arms carry their own line mapping', async ({ page }) => {
    const isCode = 'enum Color {\n  Red\n  Green\n}\n\nlet c = Color.Red\nis c {\n  Color.Red -> print("red")\n  Color.Green -> print("green")\n}\n';
    await page.click('.cm-content');
    await page.keyboard.press('Control+a');
    await page.keyboard.type(isCode);
    await page.click('.run-btn');
    await page.waitForSelector('.bytecode-line', { timeout: 15000 });
    await page.waitForTimeout(300);

    // arm at source line 7 and line 8 must map to disjoint bytecode offsets
    const armLines = page.locator('.cm-lineNumbers .cm-gutterElement');
    await armLines.filter({ hasText: /^7$/ }).click();
    await page.waitForTimeout(250);
    const offsets7 = await page.locator('.bytecode-line.is-selected')
      .evaluateAll(els => els.map(el => (el as HTMLElement).dataset.offset));
    await armLines.filter({ hasText: /^8$/ }).click();
    await page.waitForTimeout(250);
    const offsets8 = await page.locator('.bytecode-line.is-selected')
      .evaluateAll(els => els.map(el => (el as HTMLElement).dataset.offset));

    expect(offsets7.length).toBeGreaterThan(0);
    expect(offsets8.length).toBeGreaterThan(0);
    expect(offsets7.filter(o => offsets8.includes(o))).toHaveLength(0);
  });
});

test.describe('Selection vs hover highlight layers', () => {
  const code = 'var a = [5, 1, 4]\n\na.sort()\n\nprint(a[0])';

  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForSelector('.cm-content', { timeout: 10000 });
    await page.click('.cm-content');
    await page.keyboard.press('Control+a');
    await page.keyboard.type(code);
    await page.click('.run-btn');
    await page.waitForSelector('.bytecode-line', { timeout: 15000 });
    await page.waitForTimeout(300);
  });

  test('click pins a selection; hovering another line adds a hover highlight', async ({ page }) => {
    // Select line 3 by clicking its gutter
    await page.locator('.cm-lineNumbers .cm-gutterElement').filter({ hasText: /^3$/ }).click();
    const selectedOffsets = await page.locator('.bytecode-line.is-selected')
      .evaluateAll(els => els.map(el => (el as HTMLElement).dataset.offset));
    expect(selectedOffsets.length).toBeGreaterThan(0);

    // Hover line 1 in the editor content — hover highlight appears while
    // the pinned selection stays
    const editor = await page.locator('.editor-container').boundingBox();
    await page.mouse.move(editor.x + 120, editor.y + 10);
    await page.waitForTimeout(200);
    const layers = await page.evaluate(() => ({
      selected: [...document.querySelectorAll('.bytecode-line.is-selected')].map(el => (el as HTMLElement).dataset.offset),
      hovered: [...document.querySelectorAll('.bytecode-line.is-hover')].map(el => (el as HTMLElement).dataset.offset),
    }));
    expect(layers.hovered.length).toBeGreaterThan(0);
    expect(layers.selected).toEqual(selectedOffsets);
    const overlap = layers.hovered.filter(o => layers.selected!.includes(o));
    expect(overlap).toHaveLength(0);

    // Distinct colors: sample computed backgrounds of one row from each layer
    const colors = await page.evaluate(() => {
      const h = document.querySelector('.bytecode-line.is-hover');
      const s = document.querySelector('.bytecode-line.is-selected');
      return { hover: getComputedStyle(h!).backgroundColor, selected: getComputedStyle(s!).backgroundColor };
    });
    expect(colors.hover).not.toBe(colors.selected);

    // Move hover away — hover highlight clears, selection persists
    await page.mouse.move(editor.x + 40, editor.y - 60);
    await page.waitForTimeout(250);
    const after = await page.evaluate(() => ({
      selected: document.querySelectorAll('.bytecode-line.is-selected').length,
      hovered: document.querySelectorAll('.bytecode-line.is-hover').length,
    }));
    expect(after.selected).toBeGreaterThan(0);
    expect(after.hovered).toBe(0);

    // Clicking a new line moves the pinned selection
    await page.locator('.cm-lineNumbers .cm-gutterElement').filter({ hasText: /^5$/ }).click();
    await page.waitForTimeout(250);
    const newOffsets = await page.locator('.bytecode-line.is-selected')
      .evaluateAll(els => els.map(el => (el as HTMLElement).dataset.offset));
    expect(newOffsets).not.toEqual(selectedOffsets);
  });

  test('clicking editor content (not just gutter) also selects', async ({ page }) => {
    // Click in the middle of the first code line's text
    const editor = await page.locator('.editor-container').boundingBox();
    await page.mouse.click(editor.x + 140, editor.y + 11);
    await page.waitForTimeout(250);
    const selected = await page.locator('.bytecode-line.is-selected').count();
    expect(selected).toBeGreaterThan(0);
  });
});
