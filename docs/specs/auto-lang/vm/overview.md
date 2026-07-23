# vm（AutoVM）

> **Status**: partial（骨架待蒸馏，Phase 1）

## 职责

AutoVM 字节码虚拟机：ABT/字节码 codegen、执行引擎、调试器、FFI、泛型实例化。
是 UI 应用与多数示例的默认执行后端，与 a2r 存在语义一致性要求（见 docs/conformance/）。

## 关键入口

- `crates/auto-lang/src/vm.rs`、`vm/`（abt/codegen/engine/debugger/ffi/generic）
- `crates/auto-lang/src/autovm_*.rs`

## 已知坑

- UI bug 先降级为 VM 脚本复现（plan-341 方法论）。
- 与 a2r 的行为漂移需对照 `docs/conformance/` 与 plan-242 gap tracker。

## 蒸馏来源（Phase 1）

- `docs/design/05-vm-runtime.md`
- `docs/plan-reports/07-vm-runtime.md`
