// AUTO-GENERATED minimal typed runtime (PLAN-892 AX-A) — DO NOT EDIT.
// Checked i32 primitives with named traps + authorized trace marks.
#![allow(dead_code)]

/// Named Auto trap reasons (plan §5.1): add/sub/mul/neg overflow,
/// division/remainder by zero, MIN/-1 division and MIN%-1 remainder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoTrap { Overflow, DivByZero, DivOverflow, RemByZero, RemOverflow }

impl AutoTrap {
    pub fn name(self) -> &'static str {
        match self {
            AutoTrap::Overflow => "Overflow",
            AutoTrap::DivByZero => "DivByZero",
            AutoTrap::DivOverflow => "DivOverflow",
            AutoTrap::RemByZero => "RemByZero",
            AutoTrap::RemOverflow => "RemOverflow",
        }
    }
}

pub type A2xResult<T> = Result<T, AutoTrap>;

pub fn a2x_add(a: i32, b: i32) -> A2xResult<i32> { a.checked_add(b).ok_or(AutoTrap::Overflow) }
pub fn a2x_sub(a: i32, b: i32) -> A2xResult<i32> { a.checked_sub(b).ok_or(AutoTrap::Overflow) }
pub fn a2x_mul(a: i32, b: i32) -> A2xResult<i32> { a.checked_mul(b).ok_or(AutoTrap::Overflow) }
pub fn a2x_neg(a: i32) -> A2xResult<i32> { a.checked_neg().ok_or(AutoTrap::Overflow) }
pub fn a2x_div(a: i32, b: i32) -> A2xResult<i32> {
    if b == 0 { Err(AutoTrap::DivByZero) } else { a.checked_div(b).ok_or(AutoTrap::DivOverflow) }
}
pub fn a2x_rem(a: i32, b: i32) -> A2xResult<i32> {
    if b == 0 { Err(AutoTrap::RemByZero) } else { a.checked_rem(b).ok_or(AutoTrap::RemOverflow) }
}
