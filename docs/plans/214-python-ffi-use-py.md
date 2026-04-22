# Plan 214: AutoVM Python FFI — `use.py` 嵌入 Python 解释器

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** AutoVM 运行时通过嵌入 Python 解释器，支持 `use.py numpy::{array, dot}` 直接调用 Python 库函数，实现参数和返回值的自动转换。

**Architecture:** 类似 Plan 212 的 Rust FFI 模式，但替换为 Python 运行时。AutoVM 遇到 `use.py` 时，通过 PyO3 或 CPython C API 嵌入解释器，将 Python 函数包装为 native shim 注册到 AutoVM。

**Tech Stack:** PyO3 / CPython C API, RustFfiBridge 模式, Python virtualenv 管理

**Status:** Placeholder — 等待 Plan 212（Rust FFI E2E）完成后再详细设计。

---

## 依赖

**本计划依赖 Plan 212（Rust FFI E2E）完成。** Plan 212 建立的 FFI 基础设施（wrapper 生成、运行时加载、参数 marshaling、native shim 注册）的架构经验将直接迁移到 Python FFI。Plan 212 完成后，需对 FFI 功能进行分析，确定本计划的详细实施方案。

## 初步设计目标

1. **语法：** `use.py numpy::{array, dot}` — 导入 Python 模块函数
2. **运行时：** 嵌入 CPython 解释器（通过 PyO3 或直接 C API）
3. **类型转换：** AutoVM int/float/string/list/dict ↔ Python int/float/str/list/dict
4. **包管理：** 自动创建隔离的 virtualenv，安装 `dep numpy(version: "1.26")` 声明的依赖
5. **测试：** 端到端测试 `use.py json::{dumps, loads}` 验证往返

## 关键待定决策

- PyO3 vs 直接 CPython C API（PyO3 更安全但增加编译依赖）
- virtualenv 隔离策略（全局 vs per-project）
- Python GIL 管理（单线程 vs 多线程）
- 错误处理（Python exception → AutoVM error）
