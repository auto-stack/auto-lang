import { test, expect } from '@playwright/test';

const API_BASE = 'http://127.0.0.1:3030';

test.describe('Agent Debug HTTP API', () => {
  test('full debug lifecycle: start, breakpoints, continue, step, finish', async ({ request }) => {
    const source = `fn main() {
    let a = 1
    let b = 2
    let c = a + b
    print(c)
}`;

    // 1. Start session
    const startRes = await request.post(`${API_BASE}/api/agent-debug/start`, {
      data: { source },
    });
    expect(startRes.ok()).toBeTruthy();
    const startBody = await startRes.json();
    expect(startBody.session_id).toBeDefined();
    expect(Array.isArray(startBody.bytecode)).toBeTruthy();
    expect(startBody.bytecode.length).toBeGreaterThan(0);
    const sid = startBody.session_id;

    // 2. Check initial state — VM should be paused at first instruction
    const state1Res = await request.get(`${API_BASE}/api/agent-debug/${sid}/state`);
    expect(state1Res.ok()).toBeTruthy();
    const state1 = await state1Res.json();
    expect(state1.status).toBe('Paused');
    expect(state1.line).toBe(0);
    expect(state1.stdout).toBe('');

    // 3. Set breakpoint at line 3
    const bpRes = await request.post(`${API_BASE}/api/agent-debug/${sid}/breakpoints`, {
      data: { lines: [3] },
    });
    expect(bpRes.ok()).toBeTruthy();

    // 4. Continue — should hit breakpoint at line 3
    const cmd1Res = await request.post(`${API_BASE}/api/agent-debug/${sid}/command`, {
      data: { cmd: 'continue' },
    });
    expect(cmd1Res.ok()).toBeTruthy();
    const cmd1 = await cmd1Res.json();
    expect(cmd1.status).toBe('Paused');
    expect(cmd1.line).toBe(3);

    // 5. Step — should move to line 4
    const cmd2Res = await request.post(`${API_BASE}/api/agent-debug/${sid}/command`, {
      data: { cmd: 'step' },
    });
    expect(cmd2Res.ok()).toBeTruthy();
    const cmd2 = await cmd2Res.json();
    expect(cmd2.status).toBe('Paused');
    expect(cmd2.line).toBe(4);

    // 6. Continue — should run to completion
    const cmd3Res = await request.post(`${API_BASE}/api/agent-debug/${sid}/command`, {
      data: { cmd: 'continue' },
    });
    expect(cmd3Res.ok()).toBeTruthy();
    const cmd3 = await cmd3Res.json();
    expect(cmd3.status).toBe('Finished');
    expect(cmd3.stdout.trim()).toBe('3');
    expect(cmd3.result).toBeDefined();

    // 7. Delete session
    const delRes = await request.delete(`${API_BASE}/api/agent-debug/${sid}`);
    expect(delRes.ok()).toBeTruthy();

    // 8. Verify session is gone
    const state2Res = await request.get(`${API_BASE}/api/agent-debug/${sid}/state`);
    expect(state2Res.status()).toBe(404);
  });

  test('multiple concurrent sessions', async ({ request }) => {
    const source1 = `fn main() { print(1 + 1) }`;
    const source2 = `fn main() { print(2 + 2) }`;

    // Start two sessions concurrently
    const [res1, res2] = await Promise.all([
      request.post(`${API_BASE}/api/agent-debug/start`, { data: { source: source1 } }),
      request.post(`${API_BASE}/api/agent-debug/start`, { data: { source: source2 } }),
    ]);

    expect(res1.ok()).toBeTruthy();
    expect(res2.ok()).toBeTruthy();

    const body1 = await res1.json();
    const body2 = await res2.json();
    const sid1 = body1.session_id;
    const sid2 = body2.session_id;

    expect(sid1).not.toBe(sid2);

    // Both should be independent — run first to completion while second stays paused
    const [cmd1Res, state2Res] = await Promise.all([
      request.post(`${API_BASE}/api/agent-debug/${sid1}/command`, { data: { cmd: 'continue' } }),
      request.get(`${API_BASE}/api/agent-debug/${sid2}/state`),
    ]);

    expect(cmd1Res.ok()).toBeTruthy();
    const cmd1 = await cmd1Res.json();
    expect(cmd1.status).toBe('Finished');
    expect(cmd1.stdout.trim()).toBe('2');

    expect(state2Res.ok()).toBeTruthy();
    const state2 = await state2Res.json();
    expect(state2.status).toBe('Paused');

    // Clean up
    await request.delete(`${API_BASE}/api/agent-debug/${sid1}`);
    await request.delete(`${API_BASE}/api/agent-debug/${sid2}`);
  });

  test('unknown session returns 404', async ({ request }) => {
    const res = await request.get(`${API_BASE}/api/agent-debug/nonexistent/state`);
    expect(res.status()).toBe(404);
  });

  test('unknown command returns 400', async ({ request }) => {
    const source = `fn main() {}`;
    const startRes = await request.post(`${API_BASE}/api/agent-debug/start`, {
      data: { source },
    });
    const sid = (await startRes.json()).session_id;

    const cmdRes = await request.post(`${API_BASE}/api/agent-debug/${sid}/command`, {
      data: { cmd: 'fly' },
    });
    expect(cmdRes.status()).toBe(400);

    await request.delete(`${API_BASE}/api/agent-debug/${sid}`);
  });
});
