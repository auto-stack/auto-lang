const { chromium } = require('playwright');
const http = require('http');
const fs = require('fs');
const path = require('path');

// Simple static file server
const server = http.createServer((req, res) => {
  const filePath = path.join('D:\\autostack\\auto-lang\\website\\.vitepress\\dist', req.url === '/' ? '/index.html' : req.url);
  const ext = path.extname(filePath);
  const contentTypes = {
    '.html': 'text/html',
    '.js': 'application/javascript',
    '.css': 'text/css',
    '.json': 'application/json',
    '.png': 'image/png',
    '.svg': 'image/svg+xml',
    '.woff2': 'font/woff2',
  };
  
  fs.readFile(filePath, (err, data) => {
    if (err) {
      // Try with .html extension for SPA routes
      const htmlPath = filePath + '.html';
      fs.readFile(htmlPath, (err2, data2) => {
        if (err2) {
          res.writeHead(404);
          res.end('Not found');
        } else {
          res.writeHead(200, { 'Content-Type': 'text/html' });
          res.end(data2);
        }
      });
    } else {
      res.writeHead(200, { 'Content-Type': contentTypes[ext] || 'application/octet-stream' });
      res.end(data);
    }
  });
});

async function test() {
  server.listen(8888, '127.0.0.1');
  console.log('Server started on http://127.0.0.1:8888');
  
  const browser = await chromium.launch({ headless: true });
  const context = await browser.newContext({ viewport: { width: 1280, height: 800 } });
  const page = await context.newPage();
  
  // Log console messages
  page.on('console', msg => console.log('PAGE CONSOLE:', msg.type(), msg.text()));
  page.on('pageerror', err => console.log('PAGE ERROR:', err.message));
  
  try {
    // Navigate to UI Gallery page
    await page.goto('http://127.0.0.1:8888/ui-gallery.html', { waitUntil: 'networkidle' });
    console.log('Page loaded');
    
    // Take screenshot of initial state
    await page.screenshot({ path: 'D:\\autostack\\auto-lang\\test-web-tab.png' });
    console.log('Screenshot saved: test-web-tab.png');
    
    // Check if Blocks tab exists
    const blocksTab = await page.locator('button:has-text("Blocks")');
    const count = await blocksTab.count();
    console.log('Blocks tab count:', count);
    
    if (count > 0) {
      console.log('Clicking Blocks tab...');
      await blocksTab.click();
      
      // Wait for iframe to load
      await page.waitForTimeout(3000);
      
      // Take screenshot after clicking Blocks
      await page.screenshot({ path: 'D:\\autostack\\auto-lang\\test-blocks-tab.png' });
      console.log('Screenshot saved: test-blocks-tab.png');
      
      // Check iframe src
      const iframe = await page.locator('iframe[title="Blocks"]');
      const iframeCount = await iframe.count();
      console.log('Blocks iframe count:', iframeCount);
      
      if (iframeCount > 0) {
        const src = await iframe.getAttribute('src');
        console.log('Blocks iframe src:', src);
        
        // Try to access iframe content
        const frames = page.frames();
        console.log('Total frames:', frames.length);
        for (let i = 0; i < frames.length; i++) {
          try {
            const url = frames[i].url();
            console.log(`Frame ${i} URL:`, url);
          } catch (e) {
            console.log(`Frame ${i} URL: (unable to access)`);
          }
        }
        
        // Wait a bit more and check iframe content
        await page.waitForTimeout(2000);
        
        // Try to get the iframe's inner HTML
        const innerHtml = await iframe.evaluate(el => {
          try {
            return el.contentDocument ? el.contentDocument.body.innerHTML.substring(0, 500) : 'no contentDocument';
          } catch (e) {
            return 'cross-origin: ' + e.message;
          }
        });
        console.log('Iframe innerHTML (first 500 chars):', innerHtml);
      }
    }
    
  } catch (e) {
    console.error('Test error:', e);
  } finally {
    await browser.close();
    server.close();
    console.log('Done');
  }
}

test().catch(console.error);
