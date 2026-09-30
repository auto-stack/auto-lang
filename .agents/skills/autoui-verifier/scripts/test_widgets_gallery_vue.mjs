/**
 * Verify all widgets-gallery routes in Vue mode and capture full-page evidence.
 * Route and page-title inventory is derived from src/front/app.at and pages/*.at.
 *
 * Usage:
 *   node test_widgets_gallery_vue.mjs --app-dir <widgets-gallery> --base-url http://localhost:4173 --theme dark --output report.json
 */
import fs from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

async function resolvePlaywright() {
  const possiblePaths = [
    'd:/autostack/auto-lang/packages/auto-forge-ui/node_modules/playwright/index.mjs',
    'd:/autostack/auto-os-config/node_modules/playwright/index.mjs',
    'd:/autostack/auto-down/autodown/node_modules/playwright/index.mjs',
    'd:/autostack/auto-lang/examples/ui/022-kanban/tests/node_modules/playwright/index.mjs',
    'playwright',
  ];
  for (const candidate of possiblePaths) {
    try {
      return (await import(candidate.includes(':') ? pathToFileURL(candidate).href : candidate)).chromium;
    } catch {}
  }
  throw new Error('Playwright not found in standard node_modules locations.');
}

function readInventory(frontDir) {
  const app = fs.readFileSync(path.join(frontDir, 'app.at'), 'utf8');
  const routes = [...app.matchAll(/"(\/[^"]*)"\s*->\s*use\s+([\w-]+)/g)];
  const entries = routes.map(([, route, module]) => {
    const sourcePath = path.join(frontDir, 'pages', `${module}.at`);
    const source = fs.existsSync(sourcePath) ? fs.readFileSync(sourcePath, 'utf8') : '';
    const title = source.match(/\bh1\s+"([^"]+)"/)?.[1] ?? (module === 'index' ? 'Auto UI' : '');
    const previewCards = [...source.matchAll(/\bpreview-card\s*(?:\(|\{)/g)].length;
    const h2Count = [...source.matchAll(/\bh2\s+"/g)].length;
    return { route, module, title, preview_cards: previewCards, h2_count: h2Count, page_file: `${module}.at`, routed: true };
  });
  const specialPath = path.join(frontDir, 'pages', 'kitchen-sink.at');
  if (fs.existsSync(specialPath)) {
    const source = fs.readFileSync(specialPath, 'utf8');
    const existing = entries.find(e => e.module === 'kitchen-sink');
    const special = {
      route: '/kitchen-sink', module: 'kitchen-sink',
      title: source.match(/\bh1\s+"([^"]+)"/)?.[1] ?? 'Kitchen Sink',
      preview_cards: [...source.matchAll(/\bpreview-card\s*(?:\(|\{)/g)].length,
      h2_count: [...source.matchAll(/\bh2\s+"/g)].length,
      schema_examples: [...source.matchAll(/\bh2\s+"/g)].length,
      page_file: 'kitchen-sink.at', routed: Boolean(existing),
    };
    if (existing) Object.assign(existing, special);
    else entries.push(special);
  }
  return entries;
}

function parseArgs(argv) {
  const out = {};
  for (let i = 0; i < argv.length; i++) {
    if (argv[i].startsWith('--')) out[argv[i].slice(2)] = argv[++i];
  }
  if (!out['app-dir'] || !out['base-url'] || !out.theme) {
    throw new Error('Required: --app-dir <path> --base-url <url> --theme light|dark [--output <json>]');
  }
  return out;
}

const interactionChecks = {
  '/alertdialog': async page => {
    await page.getByRole('button', { name: 'Show Dialog' }).first().click();
    await page.getByRole('heading', { name: 'Are you absolutely sure?' }).waitFor({ state: 'visible' });
    return 'AlertDialog confirmation is visible';
  },
  '/dialog': async page => {
    await page.getByRole('button', { name: 'Open Dialog' }).first().click();
    await page.getByRole('heading', { name: 'Edit Profile' }).waitFor({ state: 'visible' });
    return 'Dialog content is visible';
  },
  '/sheet': async page => {
    await page.getByRole('button', { name: 'Open', exact: true }).first().click();
    await page.getByText('Profile content goes here.', { exact: true }).waitFor({ state: 'visible' });
    return 'Sheet content is visible';
  },
  '/hovercard': async page => {
    await page.locator('[data-auto-tag="hover-card-trigger"]').first().hover();
    await page.getByText('Vue component library', { exact: true }).waitFor({ state: 'visible' });
    return 'HoverCard content is visible';
  },
  '/popover': async page => {
    await page.getByRole('button', { name: 'Open Popover' }).first().click();
    await page.getByText('Set the dimensions for the layer.', { exact: true }).waitFor({ state: 'visible' });
    return 'Popover content is visible';
  },
  '/dropdownmenu': async page => {
    await page.getByRole('button', { name: 'Open', exact: true }).first().click();
    await page.getByRole('menuitem', { name: 'Profile' }).waitFor({ state: 'visible' });
    return 'Dropdown menu items are visible';
  },
  '/contextmenu': async page => {
    await page.locator('[data-auto-tag="context-menu-trigger"]').first().click({ button: 'right' });
    await page.getByRole('menuitem', { name: 'Back' }).waitFor({ state: 'visible' });
    return 'Context menu items are visible';
  },
  '/collapsible': async page => {
    await page.getByText('Can I use this in my project?', { exact: true }).click();
    await page.getByText('Yes. Free to use for personal and commercial projects. No attribution required.', { exact: true }).waitFor({ state: 'visible' });
    return 'Collapsible content is visible';
  },
  '/select': async page => {
    await page.locator('[data-auto-tag="select-trigger"]').first().click();
    await page.getByRole('option', { name: 'Apple', exact: true }).waitFor({ state: 'visible' });
    return 'Select options are visible';
  },
  '/tabs': async page => {
    await page.getByRole('tab', { name: 'Password' }).click();
    await page.getByText('Change your password here.', { exact: true }).waitFor({ state: 'visible' });
    return 'Password tab content is visible';
  },
  '/datepicker': async page => {
    await page.getByRole('button', { name: 'Pick a date' }).first().click();
    await page.getByRole('heading', { name: /Event Date/ }).waitFor({ state: 'visible' });
    return 'Date picker calendar is visible';
  },
  '/tooltip': async page => {
    await page.getByRole('button', { name: 'Hover me' }).hover();
    await page.locator('[role="tooltip"]').getByText('Add to library', { exact: true }).waitFor({ state: 'visible' });
    return 'Tooltip text is visible';
  },
  '/navigationmenu': async page => {
    await page.getByRole('button', { name: 'Components' }).hover();
    await page.getByText('Buttons and forms', { exact: true }).waitFor({ state: 'visible' });
    return 'NavigationMenu panel is visible';
  },
  '/drawer': async page => {
    await page.getByRole('button', { name: 'Open Drawer' }).click();
    await page.getByRole('heading', { name: 'Edit Profile' }).waitFor({ state: 'visible' });
    return 'Drawer panel is visible';
  },
  '/kitchen-sink': async page => {
    await page.getByRole('button', { name: 'Open dialog', exact: true }).first().click();
    await page.getByText('Dialog content sample', { exact: true }).waitFor({ state: 'visible' });
    return 'Kitchen Sink overlay example is visible';
  },
};

const args = parseArgs(process.argv.slice(2));
const appDir = path.resolve(args['app-dir']);
const frontDir = path.join(appDir, 'src', 'front');
const inventoryAll = readInventory(frontDir);
const inventory = args['only-route']
  ? inventoryAll.filter(entry => args['only-route'].split(',').includes(entry.route))
  : inventoryAll;
const screenshotDir = path.join(frontDir, 'tests', 'screenshots');
fs.mkdirSync(screenshotDir, { recursive: true });
const chromium = await resolvePlaywright();
const browser = await chromium.launch({ headless: true, channel: 'msedge' }).catch(() => chromium.launch({ headless: true }));
const context = await browser.newContext({ viewport: { width: 1440, height: 900 }, colorScheme: args.theme });
const page = await context.newPage();
page.setDefaultTimeout(7000);
const results = [];
let currentRoute = '';
const pageErrors = [];
const consoleErrors = [];
const consoleWarnings = [];
const badResponses = [];
page.on('pageerror', err => pageErrors.push({ route: currentRoute, message: err.stack ?? String(err) }));
page.on('response', response => {
  if (response.status() >= 400 && !/\/favicon\.ico(?:\?|$)/i.test(response.url())) {
    badResponses.push({ route: currentRoute, message: `${response.status()} ${response.url()}` });
  }
});
page.on('console', msg => {
  if (msg.type() === 'error') consoleErrors.push({ route: currentRoute, message: msg.text() });
  if (msg.type() === 'warning') consoleWarnings.push({ route: currentRoute, message: msg.text() });
});

try {
  for (const entry of inventory) {
    currentRoute = entry.route;
    const record = { ...entry, theme: args.theme, failures: [], screenshots: [] };
    if (!entry.routed) {
      record.failures.push('missing source route');
      results.push(record);
      continue;
    }
    const url = `${args['base-url'].replace(/\/$/, '')}/#${entry.route}`;
    try {
      await page.goto(url, { waitUntil: 'domcontentloaded', timeout: 15000 });
      await page.waitForFunction(
        ({ route, title }) => decodeURIComponent(location.hash.slice(1)) === route
          && document.querySelector('h1')?.textContent?.trim() === title,
        { route: entry.route, title: entry.title },
        { timeout: 12000 },
      );
      await page.waitForTimeout(180);
      const title = (await page.locator('h1').first().innerText()).trim();
      record.live_title = title;
      if (title !== entry.title) record.failures.push(`title mismatch: expected=${JSON.stringify(entry.title)} live=${JSON.stringify(title)}`);
      const liveH2Count = await page.locator(`h2[data-auto-tag="h2"][data-auto-src="${entry.module}"]`).count();
      record.live_h2_count = liveH2Count;
      if (entry.module !== 'index' && liveH2Count !== entry.h2_count) {
        record.failures.push(`section heading count mismatch: source=${entry.h2_count} live=${liveH2Count}`);
      }
      record.live_auto_nodes = await page.locator('[data-auto-id]').count();

      const visiblePreviews = await page.evaluate(() => {
        const buttons = [...document.querySelectorAll('button')]
          .filter(button => button.textContent.trim() === 'Auto');
        return buttons.map((button, index) => {
          let wrapper = button.parentElement?.parentElement?.parentElement ?? null;
          while (wrapper && !(wrapper.classList.contains('overflow-hidden') && wrapper.classList.contains('border'))) {
            wrapper = wrapper.parentElement;
          }
          const sample = wrapper?.firstElementChild ?? null;
          const bounds = sample?.getBoundingClientRect();
          const style = sample ? getComputedStyle(sample) : null;
          const visible = Boolean(sample && bounds && bounds.width > 0 && bounds.height > 0
            && style.display !== 'none' && style.visibility !== 'hidden');
          const hasSample = Boolean(sample && (
            sample.innerText.trim()
            || sample.querySelector('svg,img,canvas,input,textarea,select,button,[role]')
            || [...sample.querySelectorAll('*')].some(node => {
              const nodeBounds = node.getBoundingClientRect();
              const nodeStyle = getComputedStyle(node);
              const paintedBackground = nodeStyle.backgroundColor !== 'rgba(0, 0, 0, 0)'
                && nodeStyle.backgroundColor !== 'transparent';
              const paintedBorder = ['Top', 'Right', 'Bottom', 'Left'].some(side =>
                parseFloat(nodeStyle[`border${side}Width`]) > 0
                  && nodeStyle[`border${side}Color`] !== 'rgba(0, 0, 0, 0)'
                  && nodeStyle[`border${side}Color`] !== 'transparent'
              );
              return nodeBounds.width > 0 && nodeBounds.height > 0
                && nodeStyle.display !== 'none' && nodeStyle.visibility !== 'hidden'
                && (paintedBackground || paintedBorder || nodeStyle.backgroundImage !== 'none'
                  || nodeStyle.boxShadow !== 'none');
            })
          ));
          return { index, visible, has_sample: hasSample, text: sample?.innerText?.trim().slice(0, 80) ?? '' };
        });
      });
      record.live_preview_cards = visiblePreviews.length;
      record.preview_samples_visible = visiblePreviews.filter(sample => sample.visible && sample.has_sample).length;
      if (visiblePreviews.length !== entry.preview_cards) {
        record.failures.push(`preview-card count mismatch: source=${entry.preview_cards} live=${visiblePreviews.length}`);
      }
      const emptyPreviews = visiblePreviews.filter(sample => !sample.visible || !sample.has_sample).map(sample => sample.index + 1);
      record.empty_preview_samples = emptyPreviews;
      if (emptyPreviews.length) record.failures.push(`preview sample(s) without visible content: ${emptyPreviews.join(', ')}`);

      if (entry.module === 'kitchen-sink') {
        const schema = await page.evaluate(() => [...document.querySelectorAll('h2[data-auto-tag="h2"][data-auto-src="kitchen-sink"]')].map(heading => {
          const sample = heading.nextElementSibling;
          const descendants = sample ? [sample, ...sample.querySelectorAll('*')] : [];
          const hasVisibleContent = descendants.some(node => {
            const rect = node.getBoundingClientRect();
            const style = getComputedStyle(node);
            return rect.width > 0 && rect.height > 0 && style.display !== 'none' && style.visibility !== 'hidden';
          });
          return { name: heading.textContent.trim(), visible: hasVisibleContent };
        }));
        record.live_schema_examples = schema.length;
        record.visible_schema_examples = schema.filter(sample => sample.visible).length;
        record.invisible_schema_examples = schema.filter(sample => !sample.visible).map(sample => sample.name);
        if (schema.length !== entry.schema_examples) {
          record.failures.push(`schema example count mismatch: source=${entry.schema_examples} live=${schema.length}`);
        }
        if (record.invisible_schema_examples.length) {
          record.failures.push(`schema examples without visible content: ${record.invisible_schema_examples.join(', ')}`);
        }
      }

      // Gallery routes use a fixed app shell with a nested content scroller;
      // fullPage screenshots otherwise capture only the first viewport.
      const scrollInfo = await page.locator('h1').first().evaluate(heading => {
        const candidates = [];
        for (let node = heading.parentElement; node; node = node.parentElement) {
          const style = getComputedStyle(node);
          if (['auto', 'scroll'].includes(style.overflowY) && node.scrollHeight > node.clientHeight + 2) {
            candidates.push(node);
          }
        }
        const scroller = candidates.sort((a, b) => a.scrollHeight - b.scrollHeight)[0] ?? document.scrollingElement;
        return { scrollHeight: scroller.scrollHeight, clientHeight: scroller.clientHeight };
      });
      const scrollHandle = await page.locator('h1').first().evaluateHandle(heading => {
        const candidates = [];
        for (let node = heading.parentElement; node; node = node.parentElement) {
          const style = getComputedStyle(node);
          if (['auto', 'scroll'].includes(style.overflowY) && node.scrollHeight > node.clientHeight + 2) {
            candidates.push(node);
          }
        }
        return candidates.sort((a, b) => a.scrollHeight - b.scrollHeight)[0] ?? document.scrollingElement;
      });
      record.scroll_height = scrollInfo.scrollHeight;
      const slug = entry.route === '/' ? 'home' : entry.route.slice(1).replaceAll('/', '-');
      const step = 700;
      const maxOffset = Math.max(0, scrollInfo.scrollHeight - scrollInfo.clientHeight);
      const offsets = [];
      for (let offset = 0; offset < maxOffset; offset += step) offsets.push(offset);
      if (!offsets.length || offsets[offsets.length - 1] !== maxOffset) offsets.push(maxOffset);
      const maxFrames = 32;
      if (offsets.length > maxFrames) {
        record.screenshots_sampled = true;
        offsets.length = 0;
        for (let frame = 0; frame < maxFrames; frame++) {
          offsets.push(Math.round(maxOffset * frame / (maxFrames - 1)));
        }
      }
      for (const offset of offsets) {
        await scrollHandle.evaluate((element, value) => element.scrollTo({ top: value, behavior: 'instant' }), offset);
        await page.waitForTimeout(60);
        const shot = path.join(screenshotDir, `p717_vue_${args.theme}_${slug}_y${String(offset).padStart(5, '0')}.png`);
        await page.screenshot({ path: shot, animations: 'disabled' });
        record.screenshots.push(shot);
      }
      if (args.interactions === 'yes' && interactionChecks[entry.route]) {
        record.interaction = await interactionChecks[entry.route](page);
        const interactionShot = path.join(screenshotDir, `p717_vue_${args.theme}_${slug}_interaction.png`);
        await page.screenshot({ path: interactionShot, animations: 'disabled' });
        record.interaction_screenshot = interactionShot;
      }
    } catch (err) {
      record.failures.push(String(err));
    }
    results.push(record);
  }
} finally {
  await browser.close();
}

for (const row of results) {
  const routeErrors = [
    ...pageErrors,
    ...consoleErrors.filter(x => !/Failed to load resource:/.test(x.message)),
    ...badResponses,
  ].filter(x => x.route === row.route);
  for (const error of routeErrors) row.failures.push(error.message);
  const unresolved = consoleWarnings.filter(x => x.route === row.route && /failed to resolve component|unknown custom element/i.test(x.message));
  for (const warning of unresolved) row.failures.push(`unresolved component: ${warning.message}`);
  const contractWarnings = consoleWarnings.filter(x => x.route === row.route
    && /^\[(?:Vue|Vue Router) warn\]: (?:Missing required prop:|No match found for location)/.test(x.message));
  for (const warning of contractWarnings) row.failures.push(`invalid component example: ${warning.message.split(/\r?\n/)[0]}`);
}

const summary = {
  backend: 'vue', theme: args.theme, route_count: inventory.length,
  pass_count: results.filter(r => r.failures.length === 0).length,
  fail_count: results.filter(r => r.failures.length > 0).length,
  page_errors: pageErrors, console_errors: consoleErrors, bad_responses: badResponses, console_warnings: consoleWarnings,
  results,
};
if (args.output) {
  fs.mkdirSync(path.dirname(path.resolve(args.output)), { recursive: true });
  fs.writeFileSync(args.output, JSON.stringify(summary, null, 2), 'utf8');
}
console.log(JSON.stringify({ backend: summary.backend, theme: summary.theme, route_count: summary.route_count, pass_count: summary.pass_count, fail_count: summary.fail_count, page_errors: pageErrors.length, console_errors: consoleErrors.length, bad_responses: badResponses.length, console_warnings: consoleWarnings.length }));
for (const row of results) if (row.failures.length) console.log(`FAIL ${row.route} (${row.title}): ${row.failures.join('; ')}`);
for (const item of [...pageErrors, ...consoleErrors]) console.log(`BROWSER ERROR ${item.route}: ${item.message}`);
for (const item of consoleWarnings.filter(x => /failed to resolve component|unknown custom element/i.test(x.message))) console.log(`BROWSER COMPONENT WARNING ${item.route}: ${item.message}`);
process.exitCode = summary.fail_count ? 1 : 0;
