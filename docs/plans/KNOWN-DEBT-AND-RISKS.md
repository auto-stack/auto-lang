# KNOWN-DEBT-AND-RISKS — 已知技术债与风险登记簿

> **用途**：统一记录已归档计划中遗留的 workaround、一致性遗漏、架构风险和未来增强。
> 避免未来需要全扫归档计划才能找到这些隐患。
> **维护规则**：每次计划归档时的复审发现新遗留/风险，在此追加条目。
> **格式**：`[计划号] 严重度 | 类别 | 一句话描述 | 引用位置`

---

## 🔴 高风险（可能在特定场景导致 UB 或数据损坏）

| 计划 | 类别 | 描述 | 引用 |
|------|------|------|------|
| 385 | 逃逸风险 | 闭包 capture_slots 记录 creator_bp，若闭包逃逸（存入全局变量、在创建者函数返回后调用），creator_bp 指向已释放栈帧 → UB。当前无逃逸检测。常见用例（forEach 回调、直接调用）安全，因为创建者仍在栈上。 | `vm/engine.rs` Closure.capture_slots + `vm/codegen.rs:10971 compile_closure` |

---

## 🟡 一致性遗漏（功能正确但代码不干净）

| 计划 | 类别 | 描述 | 引用 |
|------|------|------|------|
| 377 | heap-aware 遗漏 | stdlib.rs 有 10 处 `push_i64(handle/server)` 未改用 `push_i64_vm`。值是 heap ID（< 2^48），实际安全，但不符合 Plan 377 的"所有 64 位值走 heap-aware"一致性目标。 | `vm/ffi/stdlib.rs:3092,3105,3115,3125,3135,3146,3478,3493,3511,3531` |
| 377 | TYPE_CAST_U64 | engine.rs:2690 的 TYPE_CAST_U64 用 `push_u64(v as u32 as u64)`，值 < 2^32 安全，但未走 heap-aware 路径。 | `vm/engine.rs:2690` |
| 340 | reduce init_val 类型 | shim_list_reduce 的 Value path 中 init_val 仍是 `pop_i32()`（而非 `pop_nv()`+`nv_to_value`）。若 reduce 初始值是 struct/str 会丢类型。常见用例（init=0/""）不受影响。 | `vm/native.rs shim_list_reduce` |

---

## 🟢 已知限制（设计决策，非 bug）

| 计划 | 类别 | 描述 | 引用 |
|------|------|------|------|
| 377 | BigInt 溢出 | virt_memory 的 push_i64/u64 在 >2^48 时 panic（快失败）。engine/native 层有 heap-aware 版本（push_i64_vm），但 virt_memory 层无 VM 访问无法堆装箱。设计如此。 | `vm/virt_memory.rs push_i64/push_u64` |
| 340 | forEach 闭包副作用 | forEach 的闭包副作用曾不生效（by-value 捕获）。Plan 385 的 capture_slots 已修复。但 forEach+Plan 385 的联动未单独测试。 | `vm/native.rs shim_list_for_each` + `vm/engine.rs capture_slots` |
| 385 | 保守 by-reference | 所有被闭包引用的外部变量都按 by-reference 处理（无 escape analysis 区分）。简单但可能有性能影响（大量变量走间接访问）。 | `vm/codegen.rs compile_closure` |
| 365 | W3 libcosmic lowering | `run_libcosmic` 的 VTree→libcosmic Element 真实 lowering 和 iced::Application 启动是 TODO（gated on libcosmic dep，Linux-only）。Windows 走 headless 委托。设计如此——按 COSMIC 组件复刻增量填充。 | `auto-cosmic/host-libcosmic/src/linux.rs` |
| 365 | W4 真实 D-Bus adapter | `LinuxNotificationsPort` 的 D-Bus signal handler 集成是 TODO（FreeDesktop Notifications 是 push API，需 COSMIC notification-daemon 组件驱动）。`LinuxPowerPort` 已实现 UPower 查询但未在 WSL2 验证。 | `auto-cosmic/ports-linux/src/linux.rs` |
| 365 | gpui Image/Grid placeholder | gpui 后端的 `View::Image` 渲染为 `[img: src]` 文本占位符，`View::Grid` 做行列分解但无原生 GPUI grid 支持。功能可用但不完整。 | `ui/gpui/auto_render.rs` + `ui/gpui/renderer.rs` |
| 365 | HostBackend Send bound | `HostBackend::run` 要求 `C::Msg: Send`（iced 的约束传播到方法级）。headless/gpui 本不需要 Send，但 GUI 消息类型按惯例都是 Send，实际无影响。 | `ui/host.rs` |
| 391 | trait impl 语法 | Auto 不支持 `impl Trait for Type` 语法（D6 仅提供清晰错误："Auto does not support trait impl syntax... Use a static fn/ext method"）。是语言设计决议，非缺陷——用 `ext Type for Spec` 表达外部 trait 实现。 | `parser.rs:4578-4585` |

---

## 📋 未来增强（非风险，记录为后续优化方向）

| 计划 | 类别 | 描述 | 引用 |
|------|------|------|------|
| 377 | opcode 合并 | ADD/ADD_F/ADD_D/ADD_U64 等变体仍保留（都单槽但未合并为单一 ADD）。合并属 plan 389。 | `vm/opcode.rs` |
| 377 | typed print 未删 | NATIVE_PRINT_I32/F32/F64/U64 仍保留为显式入口（print 路由已统一到 PRINT_UNIFIED，但 native 本身未删）。 | `vm/native_catalog.rs` |
| 340 | remove/set/insert/sort | Plan 340 只做了 HOF 方法（map/filter/find/any/all/reduce/for_each）。remove/set/insert/sort 已由 Plan 335 支持了 ListData<Value>，但未经专项测试。 | `vm/native.rs` |
| 385 | escape analysis | 未来可加 escape analysis，让不可变捕获仍走 by-value（fast path），仅可变捕获走 by-reference。 | — |
| 393 | dead-code remap 清理 | `rust.rs:5186` 的 `Expr::Bina` dispatch 块里有旧的 `"append" => Some("push_str")` remap（无守卫），但该路径是 dead code（parser 实际走 `Expr::Dot` 的 :5127/:6447 路径）。不影响功能，可选清理。 | `trans/rust.rs:5186` |
| 395 | json.decode turbofish 迁移 | `json.decode[T](text)` 可迁移为 `json.decode<Type>(text)`（rust.rs:3569 特判改为读 `generic_args`）。当前 Index hack 仍工作，auto-ai 仅 1 处使用，非阻塞。 | `trans/rust.rs:3569` |

---

| 381 | v1 限制 | Node::deserialize 只处理 props（标量字段），不含 kids（命名子块）。嵌套块反序列化留给 v2（需 field-level resolver）。覆盖 role_config 等全部用例（字段全是标量/数组）。 | `auto-val/src/de.rs:79` |

*最后更新：2026-08-07（Plan 391/393/395 归档复审后）*
