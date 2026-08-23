# plan-430 F 轮：unwrap_ok 策略 + D3 迁移 + F3 演示

- 日期：2026-08-23；worktree `429-rust-cleanup` @ `430-shim-metadata`（续 C 轮提交 f1a9316d）

## unwrap_ok（Result 解包策略，F 遗留首优先级）

真实 crate 可用面的最大闸门是 parse 族构造器全返回 `Result`。本轮落地全链路：

1. **投影**（rustdoc.rs）：`Result<T,E>` → 解包为 T 并标记 `fallible`（取第一个
   泛型实参）；Option 不解包（None 语义待例外层，v1 仍跳过）。
2. **分类双轨**：std 路径保 Option 装箱语义（Vec.get/pop 等既有行为，防丢臂）；
   三方路径 Option 跳过 + Result 走 unwrap_ok。
3. **生成器**（emit_cdylib.rs）：fallible wrapper `match` 解 Ok（成功清错误通道），
   Err 写 cdylib 线程局部 `__LAST_ERR` 并返回默认值（null/0/false）。
4. **通道导出**：`auto__last_error() -> *mut c_char`（指针归属 cdylib，VM 只拷贝）+
   `auto__clear_error()`。
5. **VM 侧**（dep_methods.rs）：fallible 方法压栈前查通道，命中转
   `VMError::RuntimeError("{Type}.{method}: {dep 错误消息}")`。

实测（semver，crates.io）：
- `Version.parse("1.2.3")` + `VersionReq.parse(">=1.0.0")` + `matches` → `1`；
  `<1.0.0` → `0`。
- 错误路径：`Version.parse("not-a-version")` →
  `RuntimeError("Version::parse failed: unexpected character 'n' while parsing major version number")`
  ——错误消息完整穿透 cdylib→marshaller→VM 三层。

## rustc 检查器剔除环（包构建鲁棒性）

一个不可编译的 wrapper 原先会弄死整包（uuid 的 `u128` 参数渲染成 `uuid::u128` 路径）。
现在 `compile_dep_methods` 构建失败时从 rustc 报错提取 `auto_*` 肇事符号
（方法符号命中剔方法；`auto__drop_<Type>` 命中剔整个类型），移入 skips（原因
"rustc check failed"）后重试，至多 4 轮；全部剔光或符号对不上才如实失败。
uuid 实测：u128 系方法被剔，包成功编译。

## D3：std Duration 迁移（最简 crate 端到端）

- std 目录加 Duration 五方法（from_secs/from_millis/from_secs_f64/as_secs/as_secs_f64），
  std-emit 补宽度收窄（u64 参数 `as u64`，与 cdylib 路径对齐）；
- 重生成 generated_std.rs → VM 实测（90 / 1.5 / **5000000000**）→ 删对应手写臂
  → 全量 3128 绿。
- **修正遗留有损截断**：旧臂 as_secs 按 i32 压栈（B3 报告标记的可疑行为），
  生成段按规则 6 走 i64 槽——5e9 秒不再变 705032704。
- 发现："Duration" 堆标签下混用两种具体类型——days/hours/seconds 构造的是
  **chrono::Duration**，from_* 是 std::time::Duration。chrono 系与 u128 返回的
  as_millis/as_micros/as_nanos 暂留手写臂。

## F3：uuid 新 crate 零手写演示

```
dep uuid(features: ["v4"])
use.rust uuid::{Uuid}
let u = Uuid.new_v4()          # 静态构造
print(u.get_version_num())     # 4
let p = Uuid.parse_str("67e55044-10b1-426f-9247-bb680e5fe0c8")  # unwrap_ok
print(p.is_nil())              # 0
print(p.get_version_num())     # 4
```

uuid 不在 BUILTIN_OPAQUE_CRATES 清单——真实 crates.io 新 crate，`dep` + `use.rust`
两行即用，零手写代码。注意 `Uuid.nil()` 会被 Auto 解析器拒（`nil` 是关键字，
"Invalid field name after dot"），用 parse_str 规避。

## marshaller 修复（uuid 实测暴露）

- **bool 返回掩码**：bool callee 只保证 al 有效，读 i64 槽的高位是垃圾——
  `is_nil()` 曾把垃圾高位带回（-1.16e18）触发 virt_memory 48 位越界 panic。
  修：`ret == 'b'` → `(r & 0xFF) != 0` + encode_bool。
- **整型压栈 heap-aware**：改用 `AutoVM::push_i64_vm`（合法大整数超 48 位内联范围
  时堆装箱，不再 panic）。
- 128 位参数分类器跳过；rustdoc `crate::` 前缀路径剥离（uuid 根重导出）。

## builtin crate 迁移阻断裁定（F1 排期依据)

对 semver/url/csv 的 legacy 足迹侦察结论：**整体迁移被两类能力缺口挡住**——

1. **字段访问**：semver major/minor/patch、csv Reader 字段都是 pub 字段非方法；
   legacy 静态表（native_catalog OPAQUE_DISPATCH_SEMVER 等 2600-2611 号 shim）
   面向字段/专用读取器实现。dep 包 v1 只有方法面。
2. **Display/trait 方法**：to_string 等 Display 方法在 trait impl 里，
   v1 只取固有 impl（trait 解析是 Plan 190 挂账项）。

因此 builtin 迁移的前置 = dep 对象字段访问通道 + trait 方法解析。
std 手写臂侧无此阻断，Duration 已示范迁法（目录 → 重生成 → 删臂 → 测试绿），
剩余 std 臂（PathBuf/File/Instant 等）按同法推进即可。

## 遗留

1. Option<T> 返回的 None 语义（is_none 集成或 None→null 值映射）；
2. dep 对象字段访问 + trait 方法解析（builtin 迁移前置）；
3. Move 语义解锁、泛型接收者 mono 提示、ABI 元数 ≤3（均同 C 轮清单）；
4. cookbook 测试若要用 dep 包路径需网络+nightly，CI 化时预生成包（F2 范畴）。
