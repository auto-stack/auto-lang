# terminal iced draw 契约（terminal-iced-draw）

> **Status**: active
> 路径：`crates/auto-lang/src/ui/terminal/iced/widget.rs` + `src/ui/iced/terminal_pixel_tests.rs`
> 技术栈：iced 0.14（wgpu 渲染 + iced_test simulator）
> 来源：PLAN-015（菜单标签根修实证）+ PLAN-634（T-01/T-02 同族收口）

## 目标与范围

- 固定 auto-term 同构 terminal widget 的**绘制期契约**：段落生命周期、
  像素级验证的环境敏感面与稳健比对策略。
- 不做：交互状态机（选中/菜单/键入,见 widget.rs 实现注记）、主题跟随
  palette 语义（PLAN-018 D10 契约）、a2r 转译规则（a2r-std）。

## draw 期段落强引用规则（SD-01,PLAN-634）

**规则：`fill_paragraph` 排队的是弱引用,段落必须有强引用存活到 flush。**

iced wgpu 渲染层在 `fill_paragraph` 时排队 `paragraph.downgrade()`
（iced_wgpu text.rs `Arc::downgrade`）,帧 flush 时 `upgrade()` 失败即
**静默丢弃**（不报错）——draw 闭包内构造的局部段落 fill 后析构,文字必然
不进像素产物（quad 为值拷贝不受影响,表现为"底色块在、字没有"）。

三处范式（widget.rs）:

| 层 | 载体 | 缓存形态 |
|---|---|---|
| 行文本 | `ROW_CACHES` | 每行 Paragraph + 内容 digest,按 terminal key 分桶 |
| 菜单标签 | `MENU_PARAS` | 静态串,`OnceLock<Vec<Para>>` 一次建 |
| badge/preedit | `PLAIN_PARAS` | 动态串,键 `(text,width)`,封顶 64 溢出清空 |

不变式:缓存条目的强引用在 flush 前不得析构;guard/Mutex 持有须跨越
fill 调用（row 缓存纪律）;动态键缓存允许封顶清空（重建只是一次小区段
shaping）,不允许悬垂。

## 像素金样环境契约（SD-02,PLAN-015 R015-F2 + 634 归因）

**漂移维度定案**:headless 像素产物的字节面绑定三张环境敏感牌——wgpu
适配器/后端枚举（`Renderer::name()` 恒 `"wgpu"`,后端翻转不换金样后缀）、
MSAA×4 合成路径、系统字体栅格（cosmic-text fontdb,`cell_w()` 实测与行
段落同解析路径）。015 收尾同日晨绿午后红、跨树复红定案环境态翻转,非代码。

**比对策略（门禁语义）**——`terminal_pixel_tests.rs`:

1. 硬门禁环境无关语义:基线墨水占比 > 0.05%、同进程双渲染逐字节一致
   （确定性噪声基线）、selection/cursor/badge/preedit 层相对基线产生
   容差差分（阈值见测试内常量,按金样校准实测标定,层缺失/在场的
   分离度 ≥2×）;
2. 金样留档 + 审计制:首跑自建;其后容差比对超 5% 预算仅告警,不再
   red 门禁（逐字节硬比对正是 015 假阳性通道）。

**受控再生成配方**:删除
`crates/auto-lang/test/ui/terminal_pixel/terminal_pixel_<name>_<kind>-*.png`
→ 重跑 `cargo test -p auto-lang --features ui-iced,iced-layout-tests
--lib terminal_pixel` 即按当前环境自建;用
`terminal_pixel_dump_golden_stats`（`--ignored --nocapture`）印证墨水/
差分量级。归因全文:docs/plans/evidence/634/pixel-golden-drift-attribution.md。

**验证约束**:像素/可见性测试一律 nextest 运行（每用例独立进程）——
裸 `cargo test` 单进程多线程共享 terminal 注册表/静态缓存,会互相污染
（634 实证:preedit 用例光标被并发用例基线重置）。

## 验证矩阵

| 改动 | 跑什么 |
|---|---|
| widget.rs 绘制段 | `cargo nextest run -p auto-lang --features ui-iced,iced-layout-tests --lib terminal_pixel`（5 用例） |
| 输入/菜单交互 | `cargo nextest run -p auto-lang --features ui-iced,iced-layout-tests --lib ui::` 定向 |
| badge/preedit 可见性回归 | 同上 terminal_pixel（负验证:局部段落旧形态 → badge/preedit 双红） |
