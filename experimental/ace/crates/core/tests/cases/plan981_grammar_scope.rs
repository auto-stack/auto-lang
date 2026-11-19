//! PLAN-981（SOURCE 批）：**grammar 裁剪的作用域不变量**。
//!
//! ## 这条测试能证明什么、不能证明什么（复查者必读）
//!
//! 冻结值是**本交付时**（commit `11545e9de`）的实测值。它防的是**将来**：任何
//! 后续改动若波及这些面，会立刻变红并报出 profile 名。
//!
//! 它**无法**证明"本交付没有改动过它们"——因为冻结动作发生在 grammar 改动
//! **之后**。而我当初那句"其余 profile 字节不变"的声明，就栽在这个时序上：
//! 实测对比 `ca88cde14` 与 HEAD，**`age-component` 确实变了**
//! （`b244ee16…` → `9c78ebbd…`），原因是它的 `age-view-member` 与新增的
//! `age-view2d/3d-member` 共用同一个 `age-node-member` alt 列表，规则体被重排。
//! 其余 13 个面（含 `age-h0`、`text-record`）**逐字节未变**。
//!
//! 也就是说：`age-component` 的那一行冻结值是"变后值"，不是"变前值"。它现在只能
//! 防止**再次**漂移。这已是既成事实，故如实记录而非假装未发生——把已变的值改回
//! 去会让当前代码红，而把声明改成"确实改了一个面"才是真实结论。
use ace_core::source::SourceProfile;

fn id_of(p: SourceProfile) -> String {
    let g = ace_core::source::cst::profile_grammar(p).expect("grammar");
    format!("{}", g.program().identity.grammar_id.0)
}

/// 这些面的 grammar id 必须与**记录的冻结值**逐字相同。
///
/// 注意范围：`age-component` 与 `age-h0` 也在列表里，所以本用例名不叫
/// "does not touch non-age profiles"——那两个是 AGE 面，且 `age-component` 的
/// 冻结值**不是** SOURCE 交付时的值（它在本批被重排过，见文件头注释）。
/// 断言消息只说"与记录的冻结值相同"，不断言"与交付前相同"，避免对那一行说谎。
#[test]
fn p981_grammar_ids_match_the_recorded_frozen_values() {
    // (profile, frozen id at SOURCE delivery)
    for (p, frozen) in [
        (
            SourceProfile::Core,
            "2296b4c559beab291d66080f912d69dc48c616fe3d6d3d13530e639a7f08f01d",
        ),
        (
            SourceProfile::Scalar,
            "fecc2ef49ea80cbee8edf07ee95d2e6dfdf955e3d90fa2baacc39fa448d5375e",
        ),
        (
            SourceProfile::Text,
            "7ba3cb83c7e96c93d8e902b95919cc87655fc79c6f96a2a8e60e7287f2022ece",
        ),
        (
            SourceProfile::Record,
            "00158ab07c08588e2381d589031b5d21977a597b2ced2bf8f31ed992efd5e346",
        ),
        // PLAN-975（T-03 MODULE text 侧，`52d2720dd`）：`r-pub`/`public-fn` 准入 `text-record`（使 `sqlite.at` 单源服务 packages/foreign 双 lane），
        // 该面 grammar id 由基线 `d820ae7fe3a0a1c02e06d5a6fa5887a8264cc350e488298ac1bd45762ec8e752` 变为下述 post-change 值；
        // 其余 13 个 profile 逐字节未变（下表全部仍与冻结值相等）。
        (
            SourceProfile::TextRecord,
            "fb3d3883b935eb4c59d29909126918ffca5692cd2b8a82fac3c4d19f40cc5dc7",
        ),
        (
            SourceProfile::EntryUnit,
            "74b9f29af50afcf90568feeac06d1306fdc406fa25990d825875b2f289854fd7",
        ),
        (
            SourceProfile::HostConsole,
            "69615a6a506ebd926dafcb7164068956f39af8183996667a1c7e0bfa5c8f61ac",
        ),
        (
            SourceProfile::Collections,
            "3e1193e9b693e0ca02b5ea3ce27578e16046a9cb9a9b80e341221e1966fe1e16",
        ),
        (
            SourceProfile::Closures,
            "3ca6a9288a5d6f516eb25a2c452c83e9d970ad8659efa3473dc32bf9dde83139",
        ),
        (
            SourceProfile::Numeric,
            "40f108201b7398b880bf0d20a7295aa63fc8acaef238a19b0b8cca5628fbc3ac",
        ),
        (
            SourceProfile::Aggregates,
            // PLAN-1026（2026-11-18）：aggregates 面的 py 训练通道语法扩是
            // 本次身份变化的唯一原因（语义差如实记录）：
            //   - 新增 `use-py-item`（`use.py <module.path>: <members>`）及
            //     `use-py-mod-tail`/`use-py-member-tail` 两条 many 尾规则，
            //     均标注 `profiles: [aggregates]`；
            //   - `use-rs-item`/`use-c-item` 的声明位 alt 与规则本体准入
            //     aggregates 面（py 通道的 ABI 对齐拼写）；
            //   - 所以只有 aggregates profile 的 grammar id 必须移动；其余
            //     邻居行的字节未变（本表逐行冻结，任何邻居漂移都会红）。
            "539a8bedcbd0ab4588b20d5fe6bcbf83ee7b304b8fa1675d1a0ce5f993d43d9b",
        ),
        (
            SourceProfile::Scheduler,
            "f26f378633dd7b3df5754ee754836cfc2759c8435df1d04ecb952712c81d0d5e",
        ),
        // R65-5（2026-11-16）：**980 WEB #6i（`b043117b8`）的语法域扩**是本次身份
        // 变化的唯一原因，语义差如实记录（不是"把哈希替成当前值"）：
        //   - `age-msg-member` 增加可选具名段 `ident(field:"msg_name")`；
        //   - `age-on-widget-handler` 增加可选参数段，并新增 `age-on-handler-params`
        //     / `age-on-handler-param-tail` 两条辅助规则；
        //   - 新段/新规则均标注 `profiles: [age-component, age-package]`，所以**只有**
        //     这两个 profile 的 grammar id 必须移动；13 个邻居行的字节未变（本表逐行
        //     冻结，任何邻居漂移都会红）。
        // 981 侧对这些新段的消费（具名集合/参数/未知消息的 A 语义）见
        // `plan981_event_a.rs`；身份基线不替代行为用例。        // AGE 自己的既有两面同样是"不得被本次打包裁剪波及"的邻居。
        // B-LANGUAGE-I（2026-11-16）：age-component 的 `msg` payload 语法修复（`Set(int)`
        // 此前在两个面都解析失败，整 node 退化成 `expected fn, found node` @0；现在用独立
        // 的 `age-msg-payload`/`age-msg-ty`，两个面都能解析 payload）。
        // resync（2026-11-16，并入 master 的 980/972 面）后该行再次变化：age-package 采纳
        // 980 富 view 时给 AGE 规则的 profile 门列表补了 `age-package`，规则文本字节变、
        // 语义不变（age-component 仍不收 view-2d/3d 与 node-ref）。
        // 此前一段历史：注意 `age-component` 的这一行是**变后值**——它在阶段 H 因与新增的
        // `age-view2d/3d-member` 共用同一个 alt 列表而被重排，基线 `ca88cde14`
        // 的值是 `b244ee1614428379afb61a756e84adfd8c947fc8584e92d7a0f04bc7bf7222e0`。
        // 保留变后值 + 在此注明，是为了让后来者看得到"这个面确实变过"，
        // 而不是被一个看似中性的数字掩盖。
        // PLAN-980 WEB #6m（2026-11-18 具名重基线）：widget 工程面语法收
        // 敛——事件键修饰符链（age-event-key，双 AGE 面）+ age-component 侧
        // 事件绑定调用形（age-event-binding）与元素括号属性行
        // （age-element-props-block）。前值 96ecc437…→1fb56340…（#6m）→f35cf593…
        //（#6o：头参数表/`;`/调用语句语法收全）随批漂移；其余 13 profile 逐字节不变（下表仍与冻结值相等）。
        // PLAN-981（D-981-GFX-SPAWN-SPELLING，2026-11-18 具名重基线）：AGE 两面
        // 的 `postfix-step` 组增 `spawn` 成员位专用臂（021 冻结拼写
        // `render2d.spawn`；保留 token 走独立臂、token 表零新增）——**只有**
        // age-component/age-package 移动（前值 f35cf593… / 0290d0ab… 之前值
        // 2d1e0547…）；其余 12 个邻居逐字节不变（下表逐行冻结，任何漂移都会红）。
        // PLAN-981（G1+G2，2026-11-18 用户裁定后同批二次具名重基线——一次重基线
        // 覆盖两处功能，13 邻居逐字节不变）：
        //   - G2：`age-on-member` 体改 `age-on-stmt`（赋值两面共用；hook-local
        //     `age-on-let`/`age-on-if` 仅 age-package 臂）——两面都动；
        //   - G1：`layer: render-only` 冻结属性行（`age-attr` 双臂包装 +
        //     `r-render-only` 单 token，仅 age-package）。
        //   前值：age-component f35cf593…→19a1376e…（spawn）→8292feae…（本批）→620a0f73…（980 view-if）→e3859aed…（980 on-if）；
        //   age-package 2d1e0547…→0290d0ab…（spawn）→5398368c…（本批）。
        //   其余 13 行（age-h0 + 12 非 AGE 邻居）本批逐字节不变——本表逐行冻结，
        //   任何邻居漂移都会在此红。
        (
            SourceProfile::AgeComponent,
            // PLAN-980 view-if 刀重基线：age-view-item 增 age-view-if/then/
            // else 三规则（两 AGE 面同开）——8292feae…→620a0f73…。
            // PLAN-980 on-if 刀：age-view-stmt 增 if 镜像臂——620a0f73…→
            // e3859aed…（master 线）。
            // PLAN-981（on-if 组件面刀，分支线 620a0f73…→ab4c2000…）：
            // `on` 体 hook-local `let`/`if` 两面同开（age-on-let/-if/-else
            // 与 age-on-stmt 双臂去 age-package 门）+ widget 形 handler 体换
            // `age-on-widget-stmt` 并集 + 分支体 `age-on-branch-stmt` 并集
            //（裸名/点前缀赋值+let+if）。
            // F-R21-3 并集第三次重基线：双线合流（980 view 方言 if 臂 +
            // 981 on 面开臂）+ age-on-widget-stmt 增 age-view-if-stmt 兜底
            // 臂 + age-on-else 增 else-if chain 臂——e3859aed…/ab4c2000…
            // 双前值均失真，按并集树实测重冻结；13 邻居逐字节不变（下表
            // 逐行冻结，任何邻居漂移都会红）。
            // PLAN-1072 刀A（组件面 style）：style-head/style-item 开
            // age-component + item 位 alt（params 步骤保持包面门控）——
            // 61799f12…→4755265c…。
            // PLAN-1072 刀A 第二刀（005 真语料）：age-event-key 增 type-kw
            // 键臂（`type: "password"` input 属性——type 是全局 token，键位
            // 直收；两面同改）——4755265c…→c8a7479d…；13 邻居逐字节不变。
            // PLAN-1072 刀B（010 真语料）：age-view-for 循环项 + view-data
            // 惰性字面量文法（模型无类型 alt）+ age-entry-value if-expr 臂
            //（条件样式值）——c8a7479d…→42afe2cb…。
            // PLAN-1072 T-03（expose 点前缀对齐）：age-expose-item / tail 兼容
            // `.name` 与裸名及可选逗号（两面同开）——42afe2cb…→fad57c5f…；
            // 13 邻居逐字节不变。
            // PLAN-105 T-02（U03 真语料，2026-11-19 具名重基线）：组件面
            // entry 值的事件绑定形改为**实参括号必填**并前置——只有
            // `onclick: .Msg(args)` 走事件绑定，`checked: .a == 0` 等比较/
            // 逻辑绑定回落 pratt（此前被裸 `.a` 贪吃，U03 元素绑定整类回滚）。
            // 仅 age-component 漂移（age-package 的事件臂本就被 profile 门
            // 排除，有效 alt 序未变）；13 邻居逐字节不变。
            "bf45f4d9b386f9484744a5e67f87acce4e7ee0b6cd6d14c7dc4525fdf687eabe",
        ),
        (
            SourceProfile::AgeH0,
            "64915bd1fc2f103e5b9c888088f53ce564c59a20bf445b1ef55dedac780f11cd",
        ),
    ] {
        assert_eq!(
            id_of(p),
            frozen,
            "{p:?}: this profile's grammar must be byte-identical to the value \
             recorded when this check was written. For age-component that recorded \
             value is POST-change (reordered in this batch), so a red here means it \
             drifted AGAIN -- not that this batch left it untouched."
        );
    }
}

/// 新面**确实**与所有邻居都不同——它不是任何既有面的别名。
#[test]
fn p981_age_package_grammar_is_not_an_alias_of_any_neighbour() {
    let mine = id_of(SourceProfile::AgePackage);
    for other in [
        SourceProfile::Core,
        SourceProfile::Scalar,
        SourceProfile::Text,
        SourceProfile::Record,
        SourceProfile::TextRecord,
        SourceProfile::EntryUnit,
        SourceProfile::HostConsole,
        SourceProfile::Collections,
        SourceProfile::Closures,
        SourceProfile::Numeric,
        SourceProfile::Aggregates,
        SourceProfile::Scheduler,
        SourceProfile::AgeComponent,
        SourceProfile::AgeH0,
    ] {
        assert_ne!(
            mine,
            id_of(other),
            "age-package must not be an alias of {other:?}"
        );
    }
}

/// 同一 profile 反复取 id 稳定（身份不是每次重算的偶然值）。
#[test]
fn p981_age_package_grammar_identity_is_stable_across_lookups() {
    let a = id_of(SourceProfile::AgePackage);
    let b = id_of(SourceProfile::AgePackage);
    assert_eq!(a, b, "grammar identity must be deterministic");
    assert_eq!(a.len(), 64, "it is a sha256 hex digest");
}

/// R59-7：age-package 的裁剪身份也被钉住（此前只钉了邻接面的 13 个冻结值）。
///
/// LANGUAGE-I 改变了它（五 view/msg payload/成员槽位），这是**有意的面扩展**：
/// 值随批次推进更新，但必须与 §6.7 的记录逐字一致，且不得在无声中漂移。
#[test]
fn p981_age_package_grammar_id_is_frozen() {
    // PLAN-980 WEB #6m（2026-11-18 具名重基线）：共享 age-event-key 规则
    // 族（事件键修饰符）进 age-package 面——前值 14561796… 随本批漂移。
    // PLAN-981（D-981-AGE-F32-SURFACE，2026-11-18 具名重基线）：`ann-ty` /
    // `ret-ty` 增 f32 注解臂（fn 参数/返回/let 注解）——**只有** age-package
    // 漂移（前值 6c1bb477…）；age-component 与 12 个非 AGE 邻居逐字节不变
    //（上表逐行冻结，任何邻居漂移都会红）。age-component 侧的 f32 走 ident
    // 拼写，不需要该臂（其 `pub fn` 本就不在面内）。
    // D-981-GFX-SPAWN-SPELLING（2026-11-18 二次重基线）：`spawn` 成员位臂
    // 进两面——前值 2d1e0547… → 本值；age-component 同批前值 f35cf593…。
    // PLAN-980 P3（2026-11-18 三次重基线，981 回签 edfa4220c）：style 声明面
    // （style-item/style-params/style-param + contextual style-head）只进
    // age-package——前值 0290d0ab… → 2d272979…（P3 单独基线）；age-component
    // 仍 19a1376e…；12 个非 AGE 邻居逐字节不变。
    // PLAN-981（G1+G2，2026-11-18 同批叠加重基线，叠在 P3 style 面之后）：
    // G2 `age-on-stmt` 双臂（两面）+ G1 `layer: render-only` 冻结属性行（仅
    // age-package）——P3 值 2d272979… → 本值；age-component 同批 19a1376e… →
    // 8292feae…（上表 AgeComponent 行）；13 邻居（age-h0 + 12 非 AGE）逐字节
    // 不变（上表逐行冻结，任何邻居漂移都会红）。
    assert_eq!(
        id_of(SourceProfile::AgePackage),
        // PLAN-980 view-if 刀重基线：c0e635d2…→30834de6…（同批双值）。
        // PLAN-980 on-if 刀：30834de6…→7497650a…（master 线）。
        // PLAN-981（on-if 组件面刀，分支线 30834de6…→a8fe5d99…）。
        // F-R21-3 并集第三次重基线：双线合流 + age-on-widget-stmt 兜底臂 +
        // age-on-else chain 臂——7497650a…/a8fe5d99… 双前值均失真，按并集
        // 树实测重冻结；age-component 同批（上表 AgeComponent 行）；13 邻居
        // 逐字节不变。
        // PLAN-1072 刀A 第二刀（age-event-key 增 type-kw 键臂，两面同改）：
        // 93cd6814…→c337685c…。
        // PLAN-1072 刀B（age-view-for/view-data/if-expr 臂两面同开）：
        // c337685c…→91c53cde…。
        // PLAN-1072 T-03（age-expose-item / tail 两面同开）：
        // 91c53cde…→06809c84…。
        // PLAN-105 T-02（U03 真语料，2026-11-19 具名重基线）：事件绑定形
        // （`age-event-binding` + args/arg 规则）开 age-package——包面真语料
        // 的 `onclick: .Msg(args)`（012/016）此前被当普通调用，verify 拒。
        // 前值 06809c84…；13 邻居逐字节不变。
        "591afe41ac82663841835a720080c447e23d26eabde453740599ffa42bf01235"
    );
}
