# AutoShell native hero correction — PLAN-713 r3

## Change

Replaced the hand-built users.json terminal illustration with the original native `website/public/apps/autoshell/ash-01.png`. Its colored prompt, syntax-highlighted `ls` command and blue directory rows now lead the product introduction. Removed the illustration's F2/F5 annotation and technology tags; existing shortcuts, original captures and script examples remain in the body.

`AutoShellPreview.vue` is shared by the EN/ZH product pages and EN/ZH v0.5 flagship sections. `EvidenceImage.vue` has an optional table preview: a CSS viewport shows the original image's top-left 1000 × 502 pixels, including all four table columns and every row. Pixels and the source image are unchanged. Enlargement displays the full 1920 × 1200 capture. Captions identify the source and preview detail. This is existing native evidence, not a newly executed shell session.

## Checks

- Whole-site VitePress build: exit 0, 180.93 seconds; command `node node_modules/vitepress/bin/vitepress.js build .` in the Plan worktree's website directory. Existing `auto` highlighting fallback and >500 kB bundle warnings remain. No Cargo suites or docs_gen (Category A).
- Browser: both languages' product and v0.5 views display the same native source; dark desktop and light English product view inspected. At 390 × 844, both product and v0.5 layouts stack without document overflow (scrollWidth 382, viewport 390). Default desktop scrollWidth 1272, viewport 1280.
- Product hero enlargement opens the full original asset. Esc closes the dialog and returns focus to the preview button. First close-key probe used the wrong accessible button name (included the decorative ×); fresh DOM observation corrected it to “关闭”, and Esc passed.
- Original ash-01/ash-2 and F1/F2/F3 body figures remain present and loaded. Lower lazy images were not forced into eager loading. Script/pipeline assets and commands were not modified; r1/r2 execution evidence still applies.
- Browser log inspection returned Vite connection messages, no error entries. `git diff --check` passed. Final edit after build only indented two CSS continuation lines; executable code and styles are identical to the built source.
- Local build required two preexisting v0.5 desktop assets. Physical copies were returned to `D:/autostack/.wt/lang-713/preview-content-backup/verification-r3`; no unrelated assets included in this change.

## Browser captures

- [Chinese native hero](autoshell-website-review-2026-09-30/zh-native-hero.jpg)
- [Chinese mobile hero](autoshell-website-review-2026-09-30/zh-native-hero-mobile.jpg)
- [Chinese v0.5 showcase](autoshell-website-review-2026-09-30/zh-v05-native-table.jpg)
- [English light hero](autoshell-website-review-2026-09-30/en-native-hero-light.jpg)

No production publication. Existing ledger writer availability blocker remains; code landing and later Plan archival are tracked separately.
