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

Playground 后端启动时自动扫描 `examples/playground-demo/*.at` 文件，通过 `GET /examples` API 返回给前端。文件按数字前缀排序，名称从文件名自动提取（如 `01-hello.at` → "Hello"）。

### API 端点

- `POST /run` — VM 执行 Auto 代码，返回 stdout
- `POST /trans` — 转译为 Rust 或 C 代码
- `GET /examples` — 获取示例列表
