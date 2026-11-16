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
            "1fd2c921bc9f54047a1744e327a96f1b39172a2a877fc74032762140f3a72f90",
        ),
        (
            SourceProfile::Scheduler,
            "f26f378633dd7b3df5754ee754836cfc2759c8435df1d04ecb952712c81d0d5e",
        ),
        // AGE 自己的既有两面同样是"不得被本次打包裁剪波及"的邻居。
        // B-LANGUAGE-I（2026-11-16）：age-component 的 `msg` payload 语法修复（`Set(int)` 此前在两个面都解析失败，整 node 退化成 `expected fn, found node` @0；现在用独立的 `age-msg-payload`/`age-msg-ty`，两个面都能解析 payload）。// resync（2026-11-16，并入 master 的 980/972 面）后该行再次变化：age-package 采纳 980 富 view 时给 AGE 规则的 profile 门列表补了 `age-package`，规则文本字节变、语义不变（age-component 仍不收 view-2d/3d 与 node-ref）。此前一段历史：// 注意：`age-component` 的这一行是**变后值**——它在阶段 H 因与新增的
        // `age-view2d/3d-member` 共用同一个 alt 列表而被重排，基线 `ca88cde14`
        // 的值是 `b244ee1614428379afb61a756e84adfd8c947fc8584e92d7a0f04bc7bf7222e0`。
        // 保留变后值 + 在此注明，是为了让后来者看得到"这个面确实变过"，
        // 而不是被一个看似中性的数字掩盖。
        (
            SourceProfile::AgeComponent,
            "96ecc4372a53665e4eb3819a51ad28c634bbdb0341e3e06ee388b95a7cf33708",
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
    assert_eq!(
        id_of(SourceProfile::AgePackage),
        "145617968fcc6d0602b1f5bcc62eaaf164ae1fdb062b09a9c79ec86eb26f5199"
    );
}