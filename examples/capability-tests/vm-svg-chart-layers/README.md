# vm-svg-chart-layers — VM 轨 svg 图表绘制分层探针

最小 barchart 静态用例（capability fixture，2026-08-25）。一屏三根元素，
逐层隔离 `auto run -r vm`（iced 原生窗口）的 chart 绘制链路：

| 元素 | 钉的层 | 缺失时指认 |
|---|---|---|
| 灰色横线（坐标轴） | svgdoc **字面量属性透传** | serialize_svg_element / svg::Handle 渲染回归 |
| 蓝柱（左，`d` 字面量） | **SVG 路径静态层** | 同上（path 路径解析） |
| 绿柱（右，`d: .p` 绑定） | **动态绑定解析层**（Init 计算赋值） | resolve_expr_to_string_with 绑定解析回归，或运行二进制早于 Plan 445 M3 修复（先 `cargo build -p auto`） |

## 运行与验证

```bash
cd examples/capability-tests/vm-svg-chart-layers
auto run -r vm          # 或 AUTOUI_MCP_PORT=9xxx auto run -r vm
```

肉眼核对三元素；或起 MCP 后 `autoui_screenshot` 截图核对。

## 关联

- 序列化格式的仓库内单测：`crates/auto-lang/src/ui/vm_bridge.rs::plan445_m3_svgdoc_dynamic_props`
- 修复与诊断记录：`docs/plans/445-024-charts.md` §M3（svgdoc 动态 props）
- 建立动机：主仓 `auto.exe` 旧于修复合入 17 分钟 → 用户实测图表区无真图、
  探针三层定位（2026-08-25）
- 完整应用级消费方：`examples/ui/024-charts/`（四类图 + 流式 + desktop_mcp 19/19）
