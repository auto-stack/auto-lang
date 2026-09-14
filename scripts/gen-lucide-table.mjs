#!/usr/bin/env node
// gen-lucide-table.mjs — 从 lucide 的图标数据生成 VM/iced 端的字形表。
//
// 背景（为什么需要它）：iced 端没有任何 lucide 数据源——字形此前是**手工抄**
// 进 `ui/iced/renderer.rs::lucide_svg` 的 `match` 表（历史上 Plan 472 加 5 条、
// Plan 618 加 1 条），所以表里只有几十个名字；而 Vue 端直接用 lucide-vue-next
// 包（1402 个）。两端因此天然分叉，且缺名在 VM 端**静默渲染成空盒**
// （债务 P537-D1）。
//
// 本脚本把 lucide 的图标数据抽成一张排序静态表，VM 端查表即可覆盖全量，
// 从而「两端同名可得同一字形」由构造保证，而不是靠人记得去补表。
//
// 用法：
//   node scripts/gen-lucide-table.mjs --src <lucide-vue-next/dist/esm/icons>
//   node scripts/gen-lucide-table.mjs            # 自动在 examples/** 里找已安装的包
//
// 数据源是本地已安装的 lucide-vue-next（生成产物里记录其版本），不联网。

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const REPO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const OUT_FILE = path.join(
  REPO_ROOT,
  "crates",
  "auto-lang",
  "src",
  "ui",
  "iced",
  "lucide_generated.rs",
);

/** 目录树里找已安装的 lucide-vue-next 的 icons 目录（pnpm 的 .pnpm 布局也覆盖）。 */
function discoverSource() {
  const roots = [path.join(REPO_ROOT, "examples"), path.join(REPO_ROOT, "packages")];
  const found = [];
  const SKIP = new Set([".git", "target", "dist", "src", "assets"]);
  const walk = (dir, depth) => {
    if (depth > 12) return;
    let entries;
    try {
      entries = fs.readdirSync(dir, { withFileTypes: true });
    } catch {
      return;
    }
    for (const e of entries) {
      if (!e.isDirectory()) continue;
      const p = path.join(dir, e.name);
      if (e.name === "lucide-vue-next") {
        const icons = path.join(p, "dist", "esm", "icons");
        if (fs.existsSync(path.join(icons, "play.js"))) found.push({ icons, pkg: p });
        continue; // 找到即停，不再下钻
      }
      if (SKIP.has(e.name) || e.name.startsWith(".")) {
        // `.pnpm` 这类点目录仍需下钻（包在它下面）
        if (!e.name.startsWith(".")) continue;
      }
      walk(p, depth + 1);
    }
  };
  for (const r of roots) walk(r, 0);
  if (found.length === 0) return null;
  // 优先版本号最高的
  const withVer = found.map((f) => {
    let v = "0.0.0";
    try {
      v = JSON.parse(fs.readFileSync(path.join(f.pkg, "package.json"), "utf8")).version;
    } catch {}
    return { ...f, v };
  });
  withVer.sort((a, b) => a.v.localeCompare(b.v, undefined, { numeric: true }));
  return withVer[withVer.length - 1];
}

/** 取 `createLucideIcon("...", [ ... ])` 里那段数组字面量（按括号配对切，不靠正则贪婪）。 */
function extractArrayLiteral(src) {
  const marker = src.indexOf("createLucideIcon(");
  if (marker < 0) return null;
  const open = src.indexOf("[", marker);
  if (open < 0) return null;
  let depth = 0;
  let inStr = null;
  for (let i = open; i < src.length; i++) {
    const c = src[i];
    if (inStr) {
      if (c === "\\") i++;
      else if (c === inStr) inStr = null;
      continue;
    }
    if (c === '"' || c === "'" || c === "`") inStr = c;
    else if (c === "[" || c === "{") depth++;
    else if (c === "]" || c === "}") {
      depth--;
      if (depth === 0) return src.slice(open, i + 1);
    }
  }
  return null;
}

const KEBAB = /-([a-z])/g;
const camelToKebab = (k) => k.replace(/[A-Z]/g, (m) => "-" + m.toLowerCase());

function parseIcon(arrayLiteral) {
  // 数组字面量是「无引号 key 的对象字面量」语法：把 key 补上引号即可 JSON.parse。
  // 图标数据里不会出现 `x:` 形态的字符串内容（路径/点集只用字母数字与 . , - 空格）。
  const jsonish = arrayLiteral.replace(
    /([{,]\s*)([A-Za-z_][A-Za-z0-9_]*)\s*:/g,
    '$1"$2":',
  );
  const arr = JSON.parse(jsonish);
  const parts = [];
  for (const node of arr) {
    if (!Array.isArray(node) || node.length < 2) continue;
    const [tag, attrs] = node;
    const keys = Object.keys(attrs).filter((k) => k !== "key");
    const rendered = keys
      .map((k) => {
        const v = attrs[k];
        const name = camelToKebab(k);
        const val = String(v).replace(/&/g, "&amp;").replace(/"/g, "&quot;");
        return ` ${name}="${val}"`;
      })
      .join("");
    parts.push(`<${tag}${rendered}/>`);
  }
  return parts.join("");
}

function main() {
  const argv = process.argv.slice(2);
  let srcDir = null;
  let version = "unknown";
  for (let i = 0; i < argv.length; i++) {
    if (argv[i] === "--src") srcDir = argv[++i];
  }
  if (srcDir) {
    // PLAN-620 T-07：--src 模式也尽量向上找 lucide-vue-next/package.json 取
    // 版本（srcDir 惯例是 <pkg>/dist/esm/icons）——否则产物版本记 "unknown"，
    // 漂移门禁只能跳过。
    let probe = path.resolve(srcDir);
    for (let i = 0; i < 6 && probe !== path.parse(probe).root; i++) {
      if (path.basename(probe) === "lucide-vue-next") {
        try {
          version = JSON.parse(fs.readFileSync(path.join(probe, "package.json"), "utf8")).version;
        } catch {}
        break;
      }
      probe = path.dirname(probe);
    }
  } else {
    const hit = discoverSource();
    if (!hit) {
      console.error(
        "[gen-lucide-table] 找不到 lucide-vue-next。先让它被安装（`auto run` 会装），\n" +
          "或显式给 --src <lucide-vue-next/dist/esm/icons>。",
      );
      process.exit(1);
    }
    srcDir = hit.icons;
    try {
      version = JSON.parse(fs.readFileSync(path.join(hit.pkg, "package.json"), "utf8")).version;
    } catch {}
    console.log(`[gen-lucide-table] source: ${path.relative(REPO_ROOT, srcDir)} (v${version})`);
  }

  const files = fs.readdirSync(srcDir).filter((f) => f.endsWith(".js") && !f.endsWith(".map"));
  const icons = [];
  const keySet = new Set();
  for (const f of files) {
    const name = f.slice(0, -3);
    const src = fs.readFileSync(path.join(srcDir, f), "utf8");
    const lit = extractArrayLiteral(src);
    if (!lit) continue;
    for (const m of lit.matchAll(/([{,]\s*)([A-Za-z_][A-Za-z0-9_]*)\s*:/g)) keySet.add(m[2]);
    let markup;
    try {
      markup = parseIcon(lit);
    } catch (e) {
      console.error(`[gen-lucide-table] 解析失败 ${name}: ${e.message}`);
      continue;
    }
    if (!markup) continue;
    if (markup.includes('"#')) {
      console.error(`[gen-lucide-table] ${name} 含 '"#'，无法用 r#"..."# 包裹`);
      process.exit(1);
    }
    icons.push([name, markup]);
  }
  icons.sort((a, b) => a[0].localeCompare(b[0]));
  console.log(`[gen-lucide-table] ${icons.length} 个图标；属性名集合: ${[...keySet].sort().join(", ")}`);

  const lines = [];
  lines.push("//! **生成文件——请勿手改**。");
  lines.push("//!");
  lines.push("//! 由 `scripts/gen-lucide-table.mjs` 从 lucide 官方图标数据抽取，");
  lines.push(`//! 数据源：lucide-vue-next v${version}（dist/esm/icons，${icons.length} 个图标）。`);
  lines.push("//!");
  lines.push("//! 为什么要全量：iced/VM 端**没有** lucide 数据源，此前字形是手工抄进");
  lines.push("//! `renderer.rs::lucide_svg` 的 match 表（只有几十个），而 Vue 端用");
  lines.push("//! lucide-vue-next 全量包 —— 缺名在 VM 端静默渲染成空盒，两端同名不同形");
  lines.push("//! （债务 P537-D1）。全量表让「同名 ⇒ 同字形」由构造保证。");
  lines.push("//!");
  lines.push("//! 重新生成：`node scripts/gen-lucide-table.mjs`");
  lines.push("");
  lines.push("/// (kebab 名, 24×24 viewBox 下的内部 markup)，按名升序，供二分查找。");
  lines.push("pub(super) static LUCIDE_ICONS: &[(&str, &str)] = &[");
  for (const [name, markup] of icons) {
    lines.push(`    ("${name}", r#"${markup}"#),`);
  }
  lines.push("];");
  lines.push("");
  // PLAN-620 T-07：机器可读的源版本——漂移门禁对拍用（"unknown" 时测试自跳过）。
  lines.push(`pub(super) const LUCIDE_SOURCE_VERSION: &str = "${version}";`);
  lines.push("");
  lines.push("/// 二分查名。命中返回字形内部 markup（24×24 坐标系，不含 `<svg>` 外壳）。");
  lines.push("pub(super) fn lookup(name: &str) -> Option<&'static str> {");
  lines.push("    LUCIDE_ICONS");
  lines.push("        .binary_search_by(|(k, _)| (*k).cmp(name))");
  lines.push("        .ok()");
  lines.push("        .map(|i| LUCIDE_ICONS[i].1)");
  lines.push("}");
  lines.push("");
  lines.push("#[cfg(test)]");
  lines.push("mod tests {");
  lines.push("    use super::*;");
  lines.push("");
  lines.push("    #[test]");
  lines.push("    fn table_is_sorted_and_nonempty() {");
  lines.push("        assert!(LUCIDE_ICONS.len() > 1000, \"全量表应覆盖 lucide 的绝大多数图标\");");
  lines.push("        for w in LUCIDE_ICONS.windows(2) {");
  lines.push("            assert!(w[0].0 < w[1].0, \"表必须按名升序（二分查找的前提）: {} !< {}\", w[0].0, w[1].0);");
  lines.push("        }");
  lines.push("    }");
  lines.push("");
  lines.push("    #[test]");
  lines.push("    fn lookups_hit_representative_names() {");
  lines.push("        // 媒体传输面（此前全缺 → 播放器只能写文字）");
  lines.push("        for n in [\"play\", \"pause\", \"skip-back\", \"skip-forward\", \"volume-1\", \"volume-2\", \"volume-x\", \"repeat\", \"maximize\", \"gauge\", \"film\"] {");
  lines.push("            assert!(lookup(n).is_some(), \"缺 {n}\");");
  lines.push("        }");
  lines.push("        // 既有示例/资产在用的常用面");
  lines.push("        for n in [\"sun\", \"moon\", \"x\", \"plus\", \"minus\", \"check\", \"chevron-left\", \"chevron-right\", \"search\", \"settings\", \"folder\", \"terminal\", \"monitor\", \"bell\", \"app-window\"] {");
  lines.push("            assert!(lookup(n).is_some(), \"缺 {n}\");");
  lines.push("        }");
  lines.push("    }");
  lines.push("");
  lines.push("    #[test]");
  lines.push("    fn lookup_misses_are_none() {");
  lines.push("        assert!(lookup(\"definitely-not-a-lucide-icon\").is_none());");
  lines.push("    }");
  lines.push("");
  lines.push("    /// PLAN-620 T-07 漂移门禁：表头记录的源版本 ↔ 本地实际安装的");
  lines.push("    /// lucide-vue-next 版本对拍——版本漂移（升包未再生）即红，指路再生命令。");
  lines.push("    /// 本地找不到安装包（CI/纯净 checkout）或生成时版本未知（--src 无 package.json）");
  lines.push("    /// 时跳过；探测根可用 LUCIDE_VUE_NEXT_ROOT 覆盖（仓内 examples 优先）。");
  lines.push("    #[test]");
  lines.push("    fn source_version_matches_installed_package() {");
  lines.push("        if LUCIDE_SOURCE_VERSION == \"unknown\" {");
  lines.push("            eprintln!(\"skipped: 生成时未记录源版本\");");
  lines.push("            return;");
  lines.push("        }");
  lines.push("        let mut roots = vec![std::path::PathBuf::from(\"examples\")];");
  lines.push("        if let Ok(env_root) = std::env::var(\"LUCIDE_VUE_NEXT_ROOT\") {");
  lines.push("            roots.push(std::path::PathBuf::from(env_root));");
  lines.push("        }");
  lines.push("        let mut installed: Option<String> = None;");
  lines.push("        for root in &roots {");
  lines.push("            for entry in walkdir::WalkDir::new(root)");
  lines.push("                .max_depth(12)");
  lines.push("                .into_iter()");
  lines.push("                .filter_map(|e| e.ok())");
  lines.push("            {");
  lines.push("                if entry.file_name().to_string_lossy() != \"lucide-vue-next\" {");
  lines.push("                    continue;");
  lines.push("                }");
  lines.push("                let pj = entry.path().join(\"package.json\");");
  lines.push("                let Ok(txt) = std::fs::read_to_string(&pj) else { continue };");
  lines.push("                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&txt) {");
  lines.push("                    if let Some(ver) = v.get(\"version\").and_then(|x| x.as_str()) {");
  lines.push("                        let better = installed.as_deref().map_or(true, |cur| {");
  lines.push("                            ver > cur");
  lines.push("                        });");
  lines.push("                        if better {");
  lines.push("                            installed = Some(ver.to_string());");
  lines.push("                        }");
  lines.push("                    }");
  lines.push("                }");
  lines.push("            }");
  lines.push("            if installed.is_some() {");
  lines.push("                break;");
  lines.push("            }");
  lines.push("        }");
  lines.push("        let Some(installed) = installed else {");
  lines.push("            eprintln!(\"skipped: 本地未找到 lucide-vue-next 安装（examples/** 与 LUCIDE_VUE_NEXT_ROOT）\");");
  lines.push("            return;");
  lines.push("        };");
  lines.push("        assert_eq!(");
  lines.push("            installed, LUCIDE_SOURCE_VERSION,");
  lines.push("            \"图标表与本地 lucide-vue-next 版本漂移——重跑 `node scripts/gen-lucide-table.mjs` 再生字形表\",");
  lines.push("        );");
  lines.push("    }");
  lines.push("}");
  lines.push("");

  fs.writeFileSync(OUT_FILE, lines.join("\n"), "utf8");
  const bytes = fs.statSync(OUT_FILE).size;
  console.log(
    `[gen-lucide-table] wrote ${path.relative(REPO_ROOT, OUT_FILE)} (${(bytes / 1024).toFixed(0)} KiB)`,
  );
}

main();
