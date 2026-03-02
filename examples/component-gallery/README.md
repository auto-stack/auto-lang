# AutoUI Component Gallery

一个类似 shadcn-vue 的组件库文档站点示例，展示 AutoUI 组件及其 Vue 转译输出。

> ⚠️ **当前状态**: 此示例使用 **Widget DSL 语法**（`widget`, `msg`, `model`, `view`, `on`），
> 需要等待 [Plan 096 (UI-IR 架构)](../../docs/plans/096-scenario-ui.md) 完成后才能编译。
>
> 当前这些文件作为 **UI 场景语法的规范示例**，展示了未来 AutoUI 的预期语法。
>
> 运行 `auto.exe vue examples/component-gallery` 将产生解析错误，因为编译器尚未支持 Widget DSL。

## Widget DSL 语法示例

这些文件使用的语法将在 UI-IR 架构实现后生效：

```auto
widget Counter {
    msg Msg { Increment, Decrement }

    model { count int = 0 }

    computed {
        doubled => .count * 2
    }

    view {
        col {
            text "Count: ${.count}"
            button (text: "+", onclick: .Increment) {}
            button (text: "-", onclick: .Decrement) {}
        }
    }

    on {
        Increment => { count = .count + 1 }
        Decrement => { count = .count - 1 }
    }
}
```

## 目录结构

```
component-gallery/
├── pac.at                      # 工作区配置
├── source/
│   └── front/
│       ├── pac.at              # 前端包配置
│       ├── app.at              # 主应用（带侧边栏导航）
│       ├── components/         # 组件定义
│       │   ├── button.at       # Button 按钮
│       │   ├── input.at        # Input 输入框
│       │   ├── text.at         # Text 文本
│       │   ├── card.at         # Card 卡片
│       │   ├── badge.at        # Badge 徽章
│       │   ├── label.at        # Label 标签
│       │   ├── accordion.at    # Accordion 折叠面板
│       │   ├── tabs.at         # Tabs 标签页
│       │   ├── code_block.at   # CodeBlock 代码块
│       │   ├── copy_button.at  # CopyButton 复制按钮
│       │   ├── sidebar.at      # Sidebar 侧边栏
│       │   └── nav_link.at     # NavLink 导航链接
│       └── pages/              # 文档页面
│           ├── index.at        # 首页
│           ├── button.at       # Button 文档
│           ├── input.at        # Input 文档
│           ├── text.at         # Text 文档
│           ├── card.at         # Card 文档
│           ├── badge.at        # Badge 文档
│           ├── label.at        # Label 文档
│           ├── accordion.at    # Accordion 文档
│           └── tabs.at         # Tabs 文档
└── generated/                  # 生成的 Vue 代码
    └── vue/
        ├── src/
        ├── App.vue
        └── main.ts
```

## 快速开始

### 前置条件
- Rust 工具链
- Node.js 20+
- npm 或 pnpm

### 生成 Vue 代码

```bash
# 从项目根目录
auto.exe vue examples/component-gallery

# 或者
cargo run --release -- vue examples/component-gallery
```

### 开发模式

```bash
cd examples/component-gallery/generated/vue
npm install
npm run dev
```

### 构建生产版本

```bash
cd examples/component-gallery/generated/vue
npm run build
```

## 组件列表

### 阶段 1 - 基础组件

| 组件 | 描述 | 属性 |
|------|------|------|
| Button | 可点击按钮 | text, onclick, variant, disabled |
| Input | 文本输入 | value, placeholder, onchange, type |
| Text | 文本显示 | content (支持插值) |
| Card | 卡片容器 | title, variant |
| Badge | 状态徽章 | text, variant |
| Label | 表单标签 | text, required, error |
| Accordion | 折叠面板 | items, defaultOpen |
| Tabs | 标签导航 | items, defaultTab |

### 文档组件

| 组件 | 描述 |
|------|------|
| CodeBlock | 语法高亮代码展示 |
| CopyButton | 剪贴板复制按钮 |
| Sidebar | 侧边栏导航 |
| NavLink | 导航链接 |

## 页面结构

每个组件文档页面包含：

1. **面包屑导航** - 显示当前位置
2. **组件描述** - 简要说明组件用途
3. **安装命令** - 如何添加组件
4. **Preview/Code 标签页** - 切换预览和代码视图
5. **Auto 源码** - 可复制的 Auto 代码
6. **Vue 转译代码** - 生成的 Vue 代码
7. **API 参考** - 属性和事件说明
8. **示例** - 各种使用场景

## 部署

项目配置了 GitHub Actions 自动部署到 GitHub Pages。

当 `examples/component-gallery/` 目录下的文件发生变更并推送到 `main` 分支时，会自动触发部署。

## 相关文档

- [Plan 103: Component Gallery 计划](../../docs/plans/103-component-gallery.md)
- [Plan 096: 场景 UI 计划](../../docs/plans/096-scenario-ui.md)
- [AURA 设计文档](../../docs/design/aura.md)
- [场景设计文档](../../docs/design/scenario.md)

## License

MIT
