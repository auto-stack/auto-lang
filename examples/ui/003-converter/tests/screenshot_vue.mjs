import { chromium } from '../../022-kanban/tests/node_modules/playwright/index.mjs';
import fs from 'fs';
import path from 'path';

async function main() {
  const outDir = 'd:/autostack/auto-lang/examples/ui/003-converter/tests/screenshots';
  fs.mkdirSync(outDir, { recursive: true });

  const browser = await chromium.launch({ headless: true });
  const context = await browser.newContext({
    viewport: { width: 1280, height: 800 },
    colorScheme: 'dark'
  });
  const page = await context.newPage();

  console.log('Navigating to http://localhost:5173...');
  await page.goto('http://localhost:5173', { waitUntil: 'networkidle' });

  // Initial screenshot
  const initialPath = path.join(outDir, 'converter_vue_initial.png');
  await page.screenshot({ path: initialPath });
  console.log('Saved Vue initial screenshot to:', initialPath);

  // Find the second input (Fahrenheit)
  const inputs = await page.locator('input').all();
  console.log(`Found ${inputs.length} inputs`);
  if (inputs.length >= 2) {
    const fahrenheitInput = inputs[1];
    await fahrenheitInput.fill('323');
    await page.waitForTimeout(500);

    const decimalPath = path.join(outDir, 'converter_vue_decimal.png');
    await page.screenshot({ path: decimalPath });
    console.log('Saved Vue decimal screenshot to:', decimalPath);

    // Read back celsius value
    const celsiusVal = await inputs[0].inputValue();
    console.log('Celsius input value is now:', celsiusVal);
  }

  await browser.close();
}

main().catch(err => {
  console.error(err);
  process.exit(1);
});
