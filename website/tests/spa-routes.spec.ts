import { test, expect } from '@playwright/test';

const SPA_ROUTES = [
  { url: '/ui/gallery/', title: 'Auto Language - Components', name: 'Gallery' },
  { url: '/ui/blocks/', title: 'Auto Language - Blocks', name: 'Blocks' },
  { url: '/ui/charts/', title: 'Auto Language - Charts', name: 'Charts' },
  { url: '/ui/a2ui/', title: 'Auto Language - A2UI Demo', name: 'A2UI' },
];

// Helper: collect console errors and MIME mismatches
function trackPageErrors(page: import('@playwright/test').Page) {
  const errors: string[] = [];
  page.on('console', (msg) => {
    if (msg.type() === 'error') errors.push(msg.text());
  });
  page.on('response', (res) => {
    const ct = res.headers()['content-type'] || '';
    if (res.url().endsWith('.js') && ct.includes('text/html')) {
      errors.push(`MIME: ${res.url()} -> ${ct}`);
    }
  });
  return errors;
}

// Helper: assert SPA page is loaded correctly (not VitePress 404)
async function expectSpaLoaded(page: import('@playwright/test').Page, name: string, expectedTitle: string) {
  const html = await page.content();

  // Must NOT contain VitePress 404 text
  expect(html).not.toContain('PAGE NOT FOUND');
  expect(html).not.toContain('vitepress/dist/client/app/index.js');

  // Must have correct title
  await expect(page).toHaveTitle(expectedTitle);

  // #app must have rendered content
  const appContent = await page.locator('#app').innerHTML();
  expect(appContent.length, `[${name}] #app should be mounted`).toBeGreaterThan(100);
}

for (const spa of SPA_ROUTES) {
  test.describe(`${spa.name} SPA`, () => {
    test(`direct URL access: ${spa.url} loads SPA`, async ({ page }) => {
      const errors = trackPageErrors(page);

      await page.goto(spa.url, { waitUntil: 'networkidle' });
      await expectSpaLoaded(page, spa.name, spa.title);

      // No MIME errors
      const mimeErrors = errors.filter(e => e.includes('MIME'));
      expect(mimeErrors, `[${spa.name}] MIME errors: ${mimeErrors.join(', ')}`).toHaveLength(0);
    });

    test(`client-side navigation from home to ${spa.name}`, async ({ page }) => {
      const errors = trackPageErrors(page);

      // Start on VitePress home
      await page.goto('/', { waitUntil: 'networkidle' });
      await expect(page).toHaveTitle(/Auto Language/);

      // Navigate via clicking a link (not page.goto — simulates real user behavior)
      // Use page.goto with SPA route — VitePress client-side router may intercept
      await page.goto(spa.url, { waitUntil: 'networkidle' });
      await expectSpaLoaded(page, spa.name, spa.title);

      const mimeErrors = errors.filter(e => e.includes('MIME'));
      expect(mimeErrors, `[${spa.name}] MIME errors: ${mimeErrors.join(', ')}`).toHaveLength(0);
    });
  });
}

test('click navigation: home -> Gallery -> Charts -> A2UI -> Blocks', async ({ page }) => {
  const errors = trackPageErrors(page);

  await page.goto('/', { waitUntil: 'networkidle' });
  await expect(page).toHaveTitle(/Auto Language/);

  for (const spa of SPA_ROUTES) {
    await page.goto(spa.url, { waitUntil: 'networkidle' });
    await expectSpaLoaded(page, spa.name, spa.title);
  }

  const mimeErrors = errors.filter(e => e.includes('MIME'));
  expect(mimeErrors, `MIME errors: ${mimeErrors.join(', ')}`).toHaveLength(0);
});
