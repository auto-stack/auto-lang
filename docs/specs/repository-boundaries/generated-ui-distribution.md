# generated-ui-distribution

> **Status**: active（PLAN-792 SD-01）
> 范围：生成式 Rust/Iced UI 工程的显式可搬动交付合同——交付布局、消费根
> patch、应用独立锁、资源 allowlist 与 producer 身份；G/H 搬仓硬门的
> 首个可执行发行面

## 交付模式与开发模式

- **开发模式**（现行，PLAN-792 零变化）：框架仓内项目 = 共享
  `examples/rust-workspace/` + workspace 继承清单（P584-D1 语义保留）；
  仓外项目 = 项目仓内 `rust-workspace/`。
- **交付模式**（`auto gen --delivery-root <dir>`，有效前端须为 rust）：
  显式可搬动受管 bundle。其 manifest **不隐式搜索 D:/autostack、不落
  编译期 `CARGO_MANIFEST_DIR` 兜底**；环境覆盖（AUTO_RUST_WORKSPACE/
  AUTO_LANG_CRATE/CARGO_TARGET_DIR 等）不进入生成物。定位待拷贝源的
  producer 根解析序 = AUTO_DELIVERY_SOURCE_ROOT → AUTO_LANG_ROOT →
  CWD 仓库漫步 → 编译期上溯（仅生成期事实，记录于 delivery-manifest）。
- 两模式共享同一编译相位（auto-man `compile_rust_ui_app`）：main.rs
  逐字节同源；sidecar/`__InitLoaded` 改写/exe_name 契约同源。

## 交付 bundle 布局（镜像生产仓根布局）

```
<delivery_root>/
  Cargo.toml            # 受管 workspace 根：members=[app+crates/*]、
                        # default-members=[app]、[workspace.dependencies]
                        # 表逐字节副本、[patch.crates-io] iced_widget（消费根）
  Cargo.lock            # 交付阶段 fresh 解析一次后冻结；消费侧 build --locked
  app/                  # 生成应用（自包含清单：相对 path 依赖 + drift pins）
  crates/<交付闭包>      # 字节级副本（仅 auto-lang Cargo.toml 投影修剪）
  crates/patches/iced_widget/  # 本地 patch 随包发行（PLAN-043 走查）
  stdlib/、docs/schema/ # 仓库根级编译期输入（include_str! 越界面）
  assets/               # 资源 allowlist（src/front 非 .at 文件 + blake3）
  delivery-manifest.json # producer 身份/闭包规模/资源/pin/投影修剪记录
```

- **镜像布局的硬理由**：auto-lang `build.rs` 以 `../../crates` 与根
  `Cargo.lock` 为编译期输入（缺失即 unwrap 红）；非镜像布局在受管 build
  阶段确定性失败。
- **交付闭包**（9 crate + 1 patch + 5 根级输入）：auto-lang（全库，含
  UI——语言源码尚未拆完，**如实披露的受管包含，不冒称纯 core/独立 UI 仓**）、
  auto-val、auto-atom、auto-lex、auto-cache、shim-metadata、auto-bindgen、
  auto-macros、a2r-std + crates/patches/iced_widget +
  stdlib/assembly-providers.json、stdlib/auto/image.{at,rs.at,vm.at}、
  docs/schema/aura.at。

## 身份与失败合同

- **单一 source 身份**：iced_widget 仅以 patch 单源出现在交付根
  `[patch.crates-io]`（不写在依赖包内）；bundle 内 package.name 全局唯一
  （同包 source 重复显式拒绝）；应用包名不得与 vendored 撞名。
- **投影修剪**（vendored auto-lang/Cargo.toml 的唯一变更）：剥交付闭包外
  optional path 依赖（autodown-core 跨仓 / auto-test-case 测试协议——
  cargo 对成员 optional path 依赖仍读 manifest，不剥则解析红）+ feature
  行剥 `dep:` 引用保留 feature 名；修剪计数不足 = producer 结构漂移判红。
- **锁与 pin**：交付应用清单对历史漂移家族（wgpu/iced 家族，来源=生成时
  producer 根 Cargo.lock 已验证版本）精确钉扎；与基础依赖同名者（iced）
  并入基础行；多槽成员（windows/windows-core 三槽）TOML 单键不可重复声明
  ——登记为 `pins_multi_slot_lock_frozen`，由冻结锁控制。779/785 记录的
  fresh-lock wgpu-hal/windows-core 漂移为时变生态漂移（2026-10-16 实测
  fresh 解析已收敛至已提交锁版本集），本轮以"独立锁+pin+冻结重放"闭合
  该失败类，不宣称生态漂移被根治。
- **确定性失败（无静默回退）**：缺资源/坏 hash 受管校验阶段判红；消费根
  漏 patch / 不兼容 pin → `cargo build --locked` 确定性红（锁需更新被拒）；
  外来非空目录（无 delivery-manifest.json 标记）显式拒绝不覆盖；producer
  缺闭包源/根级输入受管判红，**不回退主检出、不复制原 target/rlib 作
  oracle**。
- **可追溯**：delivery-manifest.json 记录 producer 源根/git HEAD/dirty/
  ui_gen 指纹、逐 crate 文件数字节数与目录 digest、资源 blake3、pin 表、
  投影修剪明细与闭包规模披露。
- **消费合同**：交付根即独立 workspace 根（自有应用 Cargo.lock）；重定位
  （A→B，不带 target/）后 `cargo build --locked` 须过；metadata 全部本地
  path source 在交付根内；运行期本地资产按 CWD 相对解析（消费侧自 assets/
  运行，开发模式对应 src/front）。生成器不自动跑 cargo——锁冻结与消费
  构建是交付管线/消费侧职责。

## 门禁

- 生成器内容门（不跑 cargo）：`cargo test -p auto-man --lib delivery -- --test-threads=1`
- 真实交付门（V01 fresh 解析+真实 build / V02 重定位 locked 重放 /
  V03 负控 / V04 开发模式保持）：`python test/fixtures/modularity/generated-ui-consumer/driver.py`
