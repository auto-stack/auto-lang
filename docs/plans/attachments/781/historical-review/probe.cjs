// Historical source-path probes; no native window or browser replay claimed.
const fs = require('node:fs');
const assert = require('node:assert/strict');
const crypto = require('node:crypto');
const root = process.cwd();
const read = p => fs.readFileSync(`${root}/${p}`, 'utf8');
const anchor = read('crates/auto-lang/src/ui/iced/scroll_anchor.rs');
assert(anchor.includes('rest.split_once(\':\')?'));
const pane = '@scrollctl:1', key = 'chapter-50';
const id = `scrollanchor:${pane}:${key}`;
const rest = id.slice('scrollanchor:'.length);
const cut = rest.indexOf(':');
const parsed = [rest.slice(0, cut), rest.slice(cut + 1)];
assert.notDeepEqual(parsed, [pane, key]);
const vue = read('crates/auto-lang/src/ui_gen/vue.rs');
const body = vue.split('{stc} {{')[1].split('{sas}')[0].trim().replace(/\}\}$/, '').replaceAll('{{', '{').replaceAll('}}', '}');
const helper = new Function('k', 'key', '__scrollCtlEl', '__sctlAnchorNote', '__sctlToChild', body);
const outerTarget = {pane: 'outer', key: 'same'};
const innerTarget = {pane: 'inner', key: 'same'};
// querySelectorAll on an outer element includes its nested pane descendants.
const outer = {querySelector: () => null, querySelectorAll: () => [outerTarget, innerTarget]};
let selected, state;
const result = helper('outer', 'same', () => outer, (h,k,s) => state=s, (e,t) => selected=t);
assert.equal(selected, innerTarget);
assert.equal(state, 'applied');
const driver = read('scripts/tools/test_reader_import_budget.py');
assert(driver.includes('verify_book_content(inst, tag, man, bid)'));
assert(driver.includes('bid, deep=False'));
const output = {
  kind: 'source-path counterexamples; not GUI/runtime acceptance',
  native_handle_roundtrip: {input:[pane,key], id, actual:parsed, expected:[pane,key]},
  vue_nested_pane: {requested:'outer/same', selected:`${selected.pane}/${selected.key}`, result, state},
  chapter_oracle: {production_driver_uses_default_deep_false:true, checked_per_format:3},
  source_sha256: Object.fromEntries([
    ['crates/auto-lang/src/ui/iced/scroll_anchor.rs',anchor],
    ['crates/auto-lang/src/ui_gen/vue.rs',vue],
    ['scripts/tools/test_reader_import_budget.py',driver]
  ].map(([p,s]) => [p,crypto.createHash('sha256').update(s).digest('hex')]))
};
console.log(JSON.stringify(output,null,2));
