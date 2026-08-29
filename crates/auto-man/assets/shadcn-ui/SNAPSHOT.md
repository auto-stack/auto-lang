# shadcn-vue component snapshot (PLAN-457)

- **Upstream**: https://shadcn-vue.com registry, `components.json` style
  `default`（与 `auto_lang::ui_gen::VueGenerator::generate_components_json`
  一致）
- **CLI**: `shadcn-vue@latest`（via `pnpm dlx`），2026-08-27 抓取
- **Scope**: 61 个组件名逐一实拍；57 个成功，3 个在 default registry 不存在
  （`auto-complete` / `input-otp` / `native-select`，见
  `vue.rs::plan_457_component_catalog_matches_bundle_or_fallback` 白名单），
  1 个（`chart` 单体条目）亦不存在但目录随 chart-* 的
  `registryDependencies` 一并落地，可直接物化。
- **Transitive**: `toggle`、`chart` 目录由依赖闭包带入（toggle-group /
  chart-* 在运行时互相 import），非冗余。
- **Baked patch**: `sonner/Sonner.vue` 已把 `CircleCheckIcon → CheckCircle`、
  `OctagonXIcon → XOctagon`、`TriangleAlertIcon → AlertTriangle`
  改名烘焙（对应 `fix_shadcn_compatibility_issues` 的兼容性改写；该函数
  保留作为 CLI 兜底路径的保险）。
- **Dependency contract**: 快照只含源码。缺口依赖 = charts 家族的
  `@unovis/vue` / `@unovis/ts`（^1.6.7），已按 Plan-442 模式接入
  `OPTIONAL_DEPS` + `VueDependencyUsage::chart`；其余外部依赖
  （reka-ui / class-variance-authority / lucide-vue-next / vaul-vue /
  embla-carousel-vue / @vueuse/core / vee-validate 等）均为既有基线。

## 重放抓取

```bash
tools/shadcn-snapshot/snapshot.sh <scratch-dir>
# 把 <scratch-dir>/src/components/ui/* 拷回本目录即可
```

升级快照时：重跑脚本 → 覆盖本目录 → 烘焙 Sonner 补丁 → 更新本文件日期。

- **AutoUI own components (not upstream snapshots)**: `nav/`（Plan 482
  NavItem/NavGroup，nav-item/nav-group 双端 class 契约的 Vue 侧镜像，
  由 auto-lang `ui::nav_contract` 单测锁定，不走 shadcn CLI 抓取）。
