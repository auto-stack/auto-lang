# Playground Demo Examples

示例 Auto 程序，用于 [Auto Playground](../../crates/auto-playground/) Web IDE。

## 示例列表

| 文件 | 功能 | VM 执行 | Rust 转译 | C 转译 |
|------|------|---------|-----------|--------|
| `01-hello.at` | Hello World | ✅ | ✅ | ✅ |
| `02-variables.at` | 变量与 F-string | ✅ | ✅ | ✅ |
| `03-functions.at` | 函数定义与调用 | ✅ | ✅ | ✅ |
| `04-fibonacci.at` | 递归斐波那契 | ✅ | ✅ | ✅ |
| `05-enums.at` | 枚举与模式匹配 | ✅ | ✅ | ✅ |
| `06-loops.at` | 循环: for/loop/break | ✅ | ✅ | ✅ |
| `07-types.at` | 类型定义与方法 | ✅ | ✅ | ✅ |
| `08-arrays.at` | 数组操作 | ✅ | ✅ | ✅ |
| `10-strings.at` | F-string 插值与多行文本 | ✅ | ✅ | ✅ |
| `11-pattern-matching.at` | `is` 模式匹配与枚举载荷 | ✅ | ✅ | ✅ |
| `12-option-result.at` | Option / Result 错误处理 | ✅ | ✅ | ⚠️ C 不支持 |
| `13-methods.at` | 类型方法、static 与 ext | ✅ | ✅ | ✅ |
| `14-generics.at` | 泛型函数 | ✅ | ✅ | ✅ |
| `15-collections.at` | HashMap / HashSet / List | ✅ | ✅ | ✅ |
| `16-sorting.at` | 数组排序与自定义比较 | ✅ | ✅ | ✅ |
| `17-text-processing.at` | 字符串方法链处理 | ✅ | ✅ | ✅ |
| `18-closures.at` | 闭包与高阶函数 | ✅ | ✅ | ✅ |
| `19-specs.at` | spec / trait 约束 | ✅ | ✅ | ✅ |
| `20-datetime.at` | 日期时间计算 | ✅ | ✅ | ⚠️ 依赖 Rust std |
| `21-encoding.at` | Base64 / hex 编解码 | ✅ | ✅ | ⚠️ 依赖 Rust std |
| `09-multi-module/` | 多模块项目：helpers 模块 | ✅ | ✅ | ✅ |
| `10-geometry/` | 多模块项目：几何图形 | ✅ | ✅ | ✅ |
| `11-bank-account/` | 多模块项目：银行账户 | ✅ | ✅ | ✅ |
| `12-todo-list/` | 多模块项目：待办清单 | ✅ | ✅ | ✅ |

## 启动 Playground

```bash
# 1. 启动后端 (Rust axum server)
cd crates/auto-playground
cargo run

# 2. 启动前端 (Vue 3 + Vite)
cd crates/auto-playground/frontend
npm install
npm run dev
```

打开 http://localhost:5173，在 Example Selector 下拉框中选择示例即可加载。

## 工作原理

Playground 后端启动时自动扫描 `examples/playground-demo/`：
- 顶层 `.at` 文件 → 单文件示例（single）
- 含 `main.at` 的子目录 → 项目示例（project），目录内所有 `.at` 文件构成模块

文件按数字前缀排序，名称从文件名/目录名自动提取（如 `01-hello.at` → "Hello"，
`09-multi-module` → "Multi-Module"）。

### API 端点

- `POST /run` — VM 执行 Auto 代码，返回 stdout；项目示例需同时提供 `project_dir`
- `POST /trans` — 转译为 Rust / C / Python / TypeScript 代码；项目示例需提供 `project_dir`
- `GET /examples` — 获取示例列表，包含单文件和项目示例

### 多模块项目说明

项目示例用于演示模块系统：入口文件必须是 `main.at`，其他 `.at` 文件作为同目录模块
通过 `use <模块名>` 引入。编辑任意模块文件后点击 Run/Trans，后端会把整套文件
写入临时目录并执行，因此非入口文件的修改也会生效。
