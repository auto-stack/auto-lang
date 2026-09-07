import { test, expect } from '@playwright/test';

// Plan 582 T16 —— /playground Notes Explorer 最小集（无后端静态路径为主）。
// 降级断言按"测试源自身是否有 /api 代理"判定：preview（静态部署）无代理→
// 降级路径可测；dev（代理 3030）→ 跳过并提示走手动冒烟。

async function originHasApiProxy(baseURL: string): Promise<boolean> {
  try {
    const res = await fetch(`${baseURL}/api/examples`, { signal: AbortSignal.timeout(1500) });
    const ct = res.headers.get('content-type') || '';
    return res.ok && ct.includes('json');
  } catch {
    return false;
  }
}

test('notes explorer renders ≥4 source groups with count badges', async ({ page }) => {
  await page.goto('/playground');
  const groups = page.locator('.nx-group');
  await expect(groups.first()).toBeVisible({ timeout: 15_000 });
  expect(await groups.count()).toBeGreaterThanOrEqual(4);
  // 计数徽章存在且为正数
  const badge = groups.first().locator('.nx-count');
  await expect(badge).toBeVisible();
  expect(Number(await badge.textContent())).toBeGreaterThan(0);
});

test('clicking a note selects it and renders a playground card', async ({ page }) => {
  await page.goto('/playground#/notes/demo/01-hello');
  const title = page.locator('.nx-note-title');
  await expect(title).toHaveText('Hello', { timeout: 15_000 });

  // demo 分组因活动笔记自动展开——点击同组另一条笔记
  await page.locator('.nx-note-btn', { hasText: 'Fibonacci' }).first().click();
  await expect(title).toHaveText('Fibonacci');
  await expect(page.locator('.playground-card')).toBeVisible();
  // 深链 hash 同步
  await expect(page).toHaveURL(/#\/notes\/demo\/04-fibonacci$/);
});

test('run action degrades to backend guidance without a backend', async ({ page }) => {
  const proxied = await originHasApiProxy('http://localhost:4173');
  test.skip(proxied, 'test origin has /api proxy — degrade path needs a backend-free origin');

  await page.goto('/playground#/notes/vm-basics/hello');
  await expect(page.locator('.nx-note-title')).toHaveText('hello', { timeout: 15_000 });
  await page.locator('.run-btn').click();
  const degrade = page.locator('.card-backend-down');
  await expect(degrade).toBeVisible({ timeout: 10_000 });
  await expect(degrade).toContainText('cargo run -p auto-playground');
  // 浏览不受影响：分组树与笔记仍可读
  const groups = page.locator('.nx-group');
  expect(await groups.count()).toBeGreaterThanOrEqual(4);
});
