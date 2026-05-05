from playwright.sync_api import sync_playwright

with sync_playwright() as p:
    browser = p.chromium.launch()
    page = browser.new_page()
    page.goto('https://a2ui-composer.ag-ui.com/gallery', wait_until='networkidle', timeout=30000)
    page.wait_for_timeout(2000)
    widgets = page.evaluate('() => { if (window.__GALLERY_WIDGETS__) return window.__GALLERY_WIDGETS__; if (window.galleryWidgets) return window.galleryWidgets; return null; }')
    print('window widgets:', widgets)
    ls = page.evaluate('() => { try { return localStorage.getItem("a2ui-gallery"); } catch(e) { return null; } }')
    print('localStorage:', ls[:500] if ls else None)
    browser.close()
