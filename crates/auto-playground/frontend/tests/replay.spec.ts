import { test, expect } from '@playwright/test';

test.describe('Debug Replay', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForSelector('.cm-content', { timeout: 10000 });
  });

  test('record debug session and export replay file', async ({ page }) => {
    const source = `fn add(a int, b int) int { a + b }
let result = add(3, 4)
print(result)`;

    // Clear editor and type source
    await page.click('.cm-content');
    await page.keyboard.press('Control+a');
    await page.keyboard.type(source);

    // Click Debug
    await page.click('.debug-btn');
    await page.waitForSelector('.debug-btn.active', { timeout: 10000 });

    // Wait for pause then click Record
    await page.waitForFunction(() => {
      const el = document.querySelector('.debug-aux-panel');
      return el && el.textContent?.includes('No variables');
    }, { timeout: 10000 });

    await page.click('.record-btn');
    await page.waitForSelector('.record-btn.recording', { timeout: 5000 });

    // Continue to run to completion
    await page.keyboard.press('F5');

    // Wait for debug to finish
    await page.waitForFunction(() => {
      const btn = document.querySelector('.debug-btn');
      return btn && !btn.classList.contains('active');
    }, { timeout: 15000 });

    // Verify result in console
    const consoleText = await page.locator('.console-main').textContent();
    expect(consoleText).toContain('7');

    // Click Save Replay — intercept download
    const [download] = await Promise.all([
      page.waitForEvent('download'),
      page.click('.save-btn'),
    ]);

    const filename = download.suggestedFilename();
    expect(filename).toMatch(/replay_\d+\.autoreplay/);
  });

  test('load replay and step through frames', async ({ page }) => {
    const recording = {
      version: 1,
      createdAt: new Date().toISOString(),
      source: 'let x = 1 + 2\nprint(x)',
      initialBreakpoints: [],
      bytecode: [
        { offset: 0, mnemonic: 'FN_PROLOG', operands: '0 16', line: undefined },
        { offset: 3, mnemonic: 'RESERVE_STACK', operands: '16', line: undefined },
        { offset: 5, mnemonic: 'CONST_1', operands: '', line: 1 },
        { offset: 6, mnemonic: 'CONST_2', operands: '', line: 1 },
        { offset: 7, mnemonic: 'ADD', operands: '', line: 1 },
        { offset: 8, mnemonic: 'STORE_LOC', operands: '0', line: 1 },
        { offset: 10, mnemonic: 'LOAD_LOC', operands: '0', line: 2 },
        { offset: 12, mnemonic: 'CALL_NAT', operands: 'print', line: 2 },
        { offset: 15, mnemonic: 'HALT', operands: '', line: undefined },
      ],
      events: [
        { type: 'state', state: { status: 'paused', line: 1, ip: 5, op: 'CONST_1', stack: [], call_stack: [], locals: [], args: [], registers: { ip: 5, bp: 0, sp: 17 }, stdout: '', stderr: '', result: null } },
        { type: 'command', cmd: 'continue' },
        { type: 'state', state: { status: 'finished', line: 0, ip: 15, op: 'HALT', stack: [], call_stack: [], locals: [{ index: 0, value: 3 }], args: [], registers: { ip: 15, bp: 0, sp: 17 }, stdout: '3\n', stderr: '', result: '' } },
      ],
    };

    // Load replay via test hook
    await page.evaluate((rec) => {
      (window as any).__loadReplayForTest__(rec);
    }, recording);

    // After loading, replay toolbar should appear
    await expect(page.locator('.replay-toolbar')).toBeVisible();

    // Verify frame info shows first frame
    const frameInfo = page.locator('.frame-info');
    await expect(frameInfo).toContainText('Frame 1 / 2');

    // Step forward
    await page.click('.replay-toolbar button[title="Step Forward (→)"]');
    await expect(frameInfo).toContainText('Frame 2 / 2');

    // Step backward
    await page.click('.replay-toolbar button[title="Step Backward (←)"]');
    await expect(frameInfo).toContainText('Frame 1 / 2');

    // Play
    await page.click('.replay-toolbar button[title="Play"]');
    await expect(page.locator('.replay-toolbar button[title="Pause"]')).toBeVisible();

    // Wait for auto-play to reach end
    await page.waitForFunction(() => {
      const info = document.querySelector('.frame-info');
      return info && info.textContent?.includes('Frame 2 / 2');
    }, { timeout: 5000 });

    // Pause
    await page.click('.replay-toolbar button[title="Pause"]');
    await expect(page.locator('.replay-toolbar button[title="Play"]')).toBeVisible();
  });
});
