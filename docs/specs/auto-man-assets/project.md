# auto-man-assets

> **Status**: active
> 路径：`crates/auto-man-assets`  | 技术栈：Rust（rust-embed + std）

Man 模板/构建器资源单一来源与纯脚手架路径规划核心（PLAN-803 从
auto-man `asset.rs` 拆出）：`assets/templates`（app/capp/clib/lib/libdemo
五族 13 文件）、`assets/builders/ghs`（3 .gpj）、`assets/builders/iar`
（4 .ew?/eww）三组嵌入资源的唯一归属，及按模板族+包名的
「源资源键 → 相对输出路径」纯计划（`plan_template_family`）。可被无
auto-lang/auto-man/auto-val/auto-gen/auto-vm/UI 闭包的外部 consumer 独立
消费（外部消费方探针 `test/fixtures/modularity/man-assets-consumer`，
闭包断言 `verify-deps.sh`）。

## 目标与范围

- 资源：`TemplateAssets`/`GhsAssets`/`IarAssets` 三个 rust-embed 根
  （子布局 templates、builders/ghs、builders/iar 原样保持；自
  `crates/auto-man/assets` git mv 逐字迁入，字节/键/顺序冻结见
  `crates/auto-man-assets/src/tests.rs` 的 SHA-256 常量表）。
- 纯计划：`plan_template_family(template, pac_name) -> Vec<PlannedFile>`
  接收包名字符串（不读 pac、不执行 Auto），按旧臂 `Templates::copy` 的
  枚举/选择/替换语义投影——iter 顺序枚举、首段等于族名的键入选、
  **余段**（去掉族前缀）对子串 `"name"` 的字面全量替换；未知族返回空。
- `PlannedFile{source_key, output_rel, bytes}`：bytes 为未求值原始字节
  （`${name}` 保持原样——内容求值与文件物化在 auto-man adapter）。
- 零输出 IO：枚举/计划不创建目录、不渲染、不写文件（无路径参数、
  无 fs 写）。
- **构建模式边界**（F803-R1-03 修正）：本 crate 随 workspace 只启用
  rust-embed `compression`（无 `debug-embed`）。默认调试态 `iter()/get()`
  为动态模式——枚举/读取宏展开期记录的**源码资源目录**（有读盘；该
  目录缺失时枚举静默为空、`get` 返回 None，同旧臂机理）；发行态（或
  显式 `debug-embed`）字节内嵌二进制，与源码目录无关。「零输出 IO」
  不覆盖调试态读盘，也不构成全模式内嵌承诺。可重复反控：
  `test/fixtures/modularity/man-assets-consumer/mode-boundary-probe.py`
  （解包副本上移开资源目录：debug 测试非零、release 零；缺资源真失败）。
- 不做：Auto 求值/模板引擎（`eval_template`/`set_global`/`repr` 链留
  auto-man `asset.rs` 门面）、pac.at 解析、exporter 生成编排
  （auto-gen 所有权）、GHS/IAR 工程结构语义（字节无语言含义地透出）、
  gallery/shadcn-ui/wm/am.at 资产（仍留 crates/auto-man/assets）。

## 依赖边界（红线）

- 依赖仅 rust-embed（+std）；dev 侧 sha2/tempfile 仅测试。
- **禁止**依赖 auto-lang / auto-ast / auto-man / auto-val / auto-gen /
  auto-vm / 任何 renderer/UI/解释器栈。闭包由 man-assets-consumer
  fixture + module-boundaries 基线钉住（man-ecosystem 域，
  `man-ecosystem-no-lang-monolith` 禁向规则适用）。
- 打包自包含：`cargo package` 清单含全部 20 资源（解包 hash 对照 =
  仓内冻结常量；缺资源即真失败，收据见 PLAN-803 T-05）。

## 冻结语义（PLAN-803 T-01 旧臂基线 `1dd123361a` 实测探针冻结；改动=行为变更须显式决策）

1. **枚举顺序** = rust-embed walkdir `sort_by_file_name`（13/3/4 键
   逐键冻结；与旧 `Templates/GHSTemplate/IARTemplate::iter()` 同机制
   同布局）。
2. **路径替换只作用余段**：`libdemo/apps/name_demo/name_demo.c` 的
   输出为 `apps/<pac>_demo/<pac>_demo.c`（族前缀不参与替换——旧臂
   `split_first` 后 `sub_path.replace("name", pac)` 逐字镜像）。
3. **`${name}` 不渲染**：计划字节即嵌入原始字节；旧臂渲染产物的
   怪异插值形态（`app("") {}`、多行整文件插值等）属 auto-man 上层
   eval_template 冻结行为，不在本层复刻或修复。
4. **未知族空结果 + 零副作用**（旧臂零匹配、不建目录）。
5. `split_first` 镜像在无 `/` 输入时与旧臂同样 panic（嵌入键恒含
   `/`，实际不可达；保留原语义不改写）。

## 分工（adapter 留 auto-man，见 [auto-man spec](../auto-man/project.md)）

- `asset.rs` 兼容门面：`Templates::copy`（Auto 求值 + 文件物化 +
  失败阶段含 unwrap panic 与先建文件再求值遗留顺序）、GHS/IAR
  inherent 访问器（AutoStr 转换）、inherent+RustEmbed 双形态
  get/iter/compressed 转发。
- scaffolding 入口 `Automan::create_by_template`、exporter 消费
  （ghs.rs/iar.rs）不变。
