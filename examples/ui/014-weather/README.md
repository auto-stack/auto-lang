# 014-weather — Weather Dashboard（PLAN-660）

中国城市天气仪表盘 demo：实况 hero、指标网格、24 小时横滑预报、5 日温度
区间条，并支持深浅主题与城市切换。数据为内置演示样本，不接真实气象 API。

设计锚点：Apple Weather × 墨迹天气。

## Concepts

- **主题契约** — 根组件声明 `dark_mode` / `accent_color`（变量名是双端运行时
  契约，Plan 458）。头部 ☀️/🌙 按钮切换深浅；`pac.at` 不写死 `theme`，跟随宿主。
- **城市选择** — 10 个中国主要城市 pill 芯片（北京/上海/广州/深圳/杭州/成都/
  西安/武汉/哈尔滨/三亚）。不用 aura `select`（`selectitem` 在 VM 臂为 none）。
- **条件感知 hero** — 天气条件驱动装饰渐变（晴 amber / 雨 blue / 雪 cyan …），
  语义 token 负责页面与卡片底色（PLAN-637 token 化 + 渐变豁免口径）。
- **结构化预报** — `for` 渲染小时卡（「现在」高亮）与 5 日行（星期/图标/条件/
  低温—渐变条—高温），替代旧版纯字符串。
- **指标网格** — 湿度/风速/体感/紫外线/AQI/能见度/气压；AQI 带等级徽标。
- **数据模块** — `src/front/weather_data.at` 纯函数目录 + `hero_grad`。

## Source layout

```
examples/ui/014-weather/
  pac.at
  src/front/types.at         # HourPoint / DayPoint / CityMeta
  src/front/weather_data.at  # 10 城 mock + 查询函数
  src/front/app.at           # UI + 状态机
```

## How to Run

```bash
cd examples/ui/014-weather
auto build              # .at → gen/front/vue
auto run                # 构建并启动 dev server（默认深色）
auto run --theme light  # 浅色主题
auto run -r vm          # VM/Iced 臂抽查（城市 pill + 主题切换）
```

Category A 验证（PLAN-660）：仅示例资产，不跑 `cargo t` / `docs_gen`。

## 主题说明

示例生态默认 Dark（`examples/ui/README.md` §主题约定）。应用内可切换；CLI
可用 `--theme light --accent ocean` 覆盖启动默认值。
