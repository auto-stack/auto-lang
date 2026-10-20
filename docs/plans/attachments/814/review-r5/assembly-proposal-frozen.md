# F814-R4-01 对账：UI 文法/宿主装配来源与合同边界（PLAN-814 R4 修复批）

## 问题陈述

R4 判定：R3-03b 采用的 `UiGrammarSource` inventory 自注册（上层被链接即
submit 生效，SDK 侧 build_dialects 惰性收集）与交付合同 r2 §5.2/V-02/
AC-02 的"不靠 inventory 偶然性""无隐式链接装配"字面冲突——尽管行为实测
通过（host/parser-only 正控均绿，且 R4 明确不声称当前样例发生随机链接
失败），该机制未经合同修订授权。

同一合同条款下，R3-01 的 `UiCompatInstaller`（宿主能力装配，覆盖不经
run 族漏斗的直达调用）已被 R3/R4 两轮复审接受并关闭——其语义边界在
SDK lib.rs 有具名注记（SDK 零 submit；上层链接即生效=确定性自注册；
纯 SDK=零提交、缺席=显式拒绝/回退）。

## 两条路线（R4 给出）

- **A. 完全显式装配**：删除一切 inventory 参与。后果：不经任何装配入口
  的直达 `Parser::from(..).with_session(UI)` 构造失去文法——这正是 R4
  自己列为**通过证据**的 parser-only 正控（"只使用旧 Parser 路径的新
  进程可解析 widget"），也是 AC-02 的旧行为保持面。路线 A 与该正控
  互斥，需砍旧 API 行为——超出 work 权限，须 needs_replan。
- **B. 显式优先 + 边界对账的兜底**（本批落地）：全部具名装配入口显式
  注入；inventory 仅作为直达构造的兜底，语义边界与已被接受的
  UiCompatInstaller 完全同构。

## 本批落地（路线 B）

1. **显式装配优先**：
   - 旧 auto-lang `install_ui_host_capabilities()`（run 族漏斗、旧公开
     入口包装、SDK 钩子回调的全部汇点）首句显式
     `set_ui_grammar(auto_ui_dialect::ui_grammar())`——含无 renderer
     构建形态（文法属语言面，与 renderer feature 无关，旧
     `ui-dialect` default 组合语义如此）。
   - auto-ui-compiler 公开生成入口（`transpile_aura`/
     `transpile_vue_aura`/`generate_component_from_file`）经
     `assemble_ui_grammar()` 显式装配。
2. **inventory 兜底保留**，SDK 侧注记改写为与 UiCompatInstaller 同构的
   语义边界声明（SDK 零 submit；纯 SDK 消费者零提交、表缺席=显式拒绝/
   回退；上层被链接即其 submit 生效——确定性注册，非 linker 偶然拉入：
   提交物是上层 crate 的显式 `inventory::submit!` 语句，不依赖任何
   静态构造器/弱符号偶然性）。
3. **回放证据**：host/parser-only/cache/autodown 四探针复跑保持 PASS
   （显式路径与兜底路径行为一致）；language-sdk-consumer 纯 SDK 负控
   （表缺席=显式拒绝）保持。

## 合同修订提案（供 T-14/T-15 裁定；不默降 AC）

**提案**：AC-02/V-02 的"无隐式链接装配"澄清为——"SDK 不依赖链接期
偶然性拉入 UI 实现（无静态构造器副作用、无 Core 硬编码 UI factory、
纯 SDK 闭包零 UI）；上层兼容包允许经显式 `inventory::submit!` 语句
注册装配器，覆盖不经装配入口的直达调用——与 PLAN-814 R3-01 已验收的
UiCompatInstaller 同一机制与同一边界"。

理由：(a) 与 R3-01 已接受的机制同构，无双重标准；(b) 直达构造正控是
AC-02 自身的旧行为保持面，路线 A 无法同时满足两者；(c) 提交物是上层
crate 源码中的显式语句，可审计、可负控（纯 SDK fixture 证明零提交）。

若裁定不接受该澄清：needs_replan 路线的具体变更是"直达
`Parser::from(..).with_session(UI)` 在未装配时从『空方言回退』改为
『硬错误』或删除该直达面"——两者都是旧 API 行为破坏，需用户授权。

## SD 增量正文（A 子集，绑定本批）

- **SD-01（language-sdk）补**：SDK 公开 `set_ui_grammar(UiGrammar)`/
  `UiGrammarSource`（inventory 收集点，SDK 零提交）与
  `set_generator_fingerprint`（None→Some 升级、首装优先、save 按出生
  身份落盘）两个装配端口的精确语义。
- **SD-02（auto-ui-compiler）补**：`assemble_ui_grammar()` 公开入口与
  三个生成入口的显式装配时序；`generator_fingerprint()` 的 None 语义
  （foreign build）。
- **SD-04/05（auto-lang/frontend）补**：Scenario::UI 内建方言的装配
  来源双层（显式安装器首句 + inventory 兜底）；表缺席=空方言回退+
  公开门面显式拒绝的诊断文本（"not assembled ... UI grammar table"）。
- **auto-parser-api 归属变更**：`UiParseContext` 自 auto-ui-dialect 迁入
  （PLAN-810 §2"不入 Core 包"裁定的修订：孤儿规则下 impl 只能落在
  trait 或类型所属 crate，SDK 无清单边要求 impl 留 SDK——唯一合法位置
  是双侧共同依赖的轻端口 crate；裁定理据保持——独立 opt-in 子 trait，
  ParseContext 实现方零强加，旧路径 re-export 不变）。canonical
  `auto-parser-api/project.md` 与 `auto-ui-dialect/design/full-grammar.md`
  的相应正文随 T-15 受管沉淀（本批不写 canonical Specs）。
- **runtime 缓存规范（PLAN-698 语义在分包后的精确化）**：指纹槽
  None→Some=升级；已装配=首装优先；save 按缓存对象出生身份落盘（不
  重盖）；load 经 cached-vs-current 比对判失效（一侧缺席=整缓存失效）；
  真实身份消费方（auto-man vue/jet、compiler api）在 load 前注入。
