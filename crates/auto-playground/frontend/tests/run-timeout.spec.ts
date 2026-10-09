import { test, expect } from '@playwright/test';

// PLAN-746 (PG-MEM-1): /api/run execution deadline — the playground server
// must return a clean, structured timeout error for runaway programs
// instead of hanging forever and leaking memory server-side.
// Regression corpus: think-python ch07 fences + plan231 quarantined loops
// (walkthrough 2026-10-09: +210/+348MB server growth per hung example).

const INFINITE = 'for true {\n}';

test.describe('Run execution deadline', () => {
  test('infinite loop returns a timeout error quickly (default 10s)', async ({ request }) => {
    const start = Date.now();
    const res = await request.post('/api/run', {
      data: { source: INFINITE },
    });
    const body = await res.json();
    const elapsed = Date.now() - start;

    expect(res.ok()).toBe(true);
    // vm_runner surfaces VM errors in the stdout field ("Error: ...").
    expect(body.result + body.stdout).toContain('Error');
    expect(body.result + body.stdout).toContain('ExecutionTimeout');
    expect(elapsed).toBeLessThan(20_000);
  });

  test('short timeout_secs is honored', async ({ request }) => {
    const start = Date.now();
    const res = await request.post('/api/run', {
      data: { source: INFINITE, timeout_secs: 2 },
    });
    const body = await res.json();
    const elapsed = Date.now() - start;

    expect(body.result + body.stdout).toContain('ExecutionTimeout');
    expect(elapsed).toBeLessThan(15_000);
  });

  test('print kwargs shape fails at compile time (no runaway execution)', async ({
    request,
  }) => {
    const res = await request.post('/api/run', {
      data: { source: 'for i in 0..3 {\n    print(i, end=" ")\n}' },
    });
    const body = await res.json();

    expect(res.ok()).toBe(true);
    expect(body.result + body.stdout).toContain('keyword arguments');
  });

  test('server stays healthy after a timeout (no runaway thread)', async ({
    request,
  }) => {
    await request.post('/api/run', { data: { source: INFINITE, timeout_secs: 1 } });
    // A normal run right after must succeed promptly.
    const res = await request.post('/api/run', {
      data: { source: 'print(1 + 2)' },
    });
    const body = await res.json();
    expect(body.stdout).toContain('3');
  });

  test('UI Run requests timeout_secs=60 (F-746-R1: slow legit examples)', async ({
    page,
  }) => {
    // 官方宿主必须显式请求 60s 上限——否则合法慢示例（parity C/sync-http
    // 族 ~10.5s）会被服务端默认 10s 截止截断。
    let runPayload: any = null;
    page.on('request', (req) => {
      if (req.url().endsWith('/api/run') && req.method() === 'POST') {
        runPayload = req.postDataJSON();
      }
    });
    await page.goto('/');
    await page.waitForSelector('.cm-content', { timeout: 10000 });
    await page.click('.run-btn');
    await page.waitForResponse(
      (res) => res.url().endsWith('/api/run') && res.status() === 200,
      { timeout: 15000 },
    );
    expect(runPayload).not.toBeNull();
    expect(runPayload.timeout_secs).toBe(60);
  });
});
