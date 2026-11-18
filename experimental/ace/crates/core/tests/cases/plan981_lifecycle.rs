//! PLAN-981（B-LANGUAGE-I / AGE-L05）：生命周期 oracle 的**不倍增**契约。
//!
//! §5 冻结："5 Tick + 3 owner Frame 周期、两个出口，Start=1 / Tick=5 / Frame=3"。
//! 本用例把"Frame/Tick 不随出口数倍增"写成机械断言：出口数只做合法性检查，
//! 不进入计数。真实宿主计数由 SESSION 接线后经 V-L-LIFE 核对。
use ace_core::source::age_package::lifecycle_budget;

/// 给定 5 Tick + 3 Frame：Start=1、Tick=5、Frame=3。
#[test]
fn p981_lifecycle_oracle_matches_the_frozen_numbers() {
    for exits in 1..=5usize {
        let b = lifecycle_budget(5, 3, exits).expect("a scheduled owner has exits");
        assert_eq!(b.starts, 1, "Start is once per logical owner");
        assert_eq!(b.ticks, 5, "Tick follows the schedule");
        assert_eq!(
            b.frames, 3,
            "Frame follows the schedule, not the exit count"
        );
    }
}

/// 同一调度下，出口数从 1 变到 5，三个计数**逐字不变**（不倍增）。
#[test]
fn p981_lifecycle_counts_do_not_scale_with_exits() {
    let one = lifecycle_budget(5, 3, 1).unwrap();
    let many = lifecycle_budget(5, 3, 5).unwrap();
    assert_eq!(one, many, "lifecycle counts must not depend on exit count");
}

/// 零出口：没有可呈现的面 → 具名拒绝（不产生 Start/Tick/Frame 计数）。
#[test]
fn p981_lifecycle_requires_a_presentation_exit() {
    let err = lifecycle_budget(5, 3, 0).expect_err("no exit means no scheduled owner");
    assert_eq!(err.code, "source.age.lifecycle", "{err:?}");
}

/// 调度次数是作者/宿主给定的：0 Tick / 0 Frame 也是合法调度（不是错误）。
#[test]
fn p981_zero_tick_or_frame_schedule_is_legal() {
    let b = lifecycle_budget(0, 0, 1).expect("zero schedule is still a schedule");
    assert_eq!(b.starts, 1);
    assert_eq!(b.ticks, 0);
    assert_eq!(b.frames, 0);
}
