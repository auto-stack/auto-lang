# Frozen proposed Spec delta


| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-media-core/project.md | UI目录的中立媒体机制→独立单源类型/共享服务、资源/寿命/支持合同 | 可独立消费 | AC-01..05 |
| SD-02 | modify | docs/specs/auto-lang/vm/overview.md | 17 native及HTTP UI直连→非UI服务/编解码、开关及stub冻结 | VM脱UI类型 | AC-01, AC-02, AC-04 |
| SD-03 | modify | docs/specs/auto-lang/ui/overview.md | 图像/媒体索引/Range机制迁core、renderer及剩余owner真实列明 | 不虚报全UI独立 | AC-01..05 |
| SD-04 | modify | docs/specs/auto-lang/ui/design/image-surface-events.md | 规则保持，registry/代次owner改core、事件与渲染拍留UI | 保护loaded/error | AC-04 |
| SD-05 | modify | docs/specs/auto-man/project.md | 媒体生成/HTTP接线消费新core，项目模型/UI工具搬迁仍未发生 | 分清809/R04边界 | AC-04, AC-05 |
| SD-06 | modify | docs/specs/repository-boundaries/generated-ui-distribution.md | 新core及codec/source闭包进入受影响交付profile | 独立可分发 | AC-05 |


## Handoff SD draft
（merge 时沉淀 canonical；本轮不私改）

- **SD-01 add `docs/specs/auto-media-core/project.md`**：crate 使命/公开面
  （image_pipeline 全 API+MediaRuntime+media_service）/资源合同（预算表、
  1GiB/400MP、TTL 双值 30s 全局/2s 常量）/生命周期合同（shutdown 终态
  化、无复活、满队列有界收敛、锁序：无用户回调/无 registry 锁内等
  worker）/发行合同（image-codec feature、闭包断言）。
- **SD-02 modify `vm/overview.md`**：17 native+HTTP 快路径消费
  auto-media-core（直调，非 UI 门面）；ID/按名注册/feature 门/stub 语义
  冻结表；控制面不搬像素/路径。
- **SD-03 modify `ui/overview.md`**：image_pipeline/media_service 迁 core、
  门面身份同一；renderer/photo_service/mpv/i18n/frame 等 UI 剩余 owner
  真实列明（不虚报全 UI 独立）。
- **SD-04 modify `ui/design/image-surface-events.md`**：规则不变
  （Ready 后单发/形参投影/换源退订/A→B→A/50ms 代次）；registry/代次/
  pending 门 owner 改 core，事件与渲染拍留 UI。
- **SD-05 modify `auto-man/project.md`**：api_gen/rust_ui 媒体生成直调
  core；生成 manifest 携带 auto-media-core（workspace/path/交付闭包）；
  项目模型/UI 工具搬迁仍未发生（809/R04 边界）。
- **SD-06 modify `repository-boundaries/generated-ui-distribution.md`**：
  DELIVERY_CRATES +auto-media-core（image/exif/blake3/fastrand 走
  registry）；受影响 profile 的 fresh/locked 合同由 792 钉住。

