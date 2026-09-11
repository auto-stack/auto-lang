//! PLAN-596 land 期修正: 回调面专用最小 fixture(独立 crate,见 Cargo.toml 注)。

/// 回调 adapter 原型宿主:Box<dyn Fn> 形参 → T5 callbacks 元数据 + adapter。
pub struct Invoker {
    _label: String,
}

impl Invoker {
    pub fn new() -> Invoker {
        Invoker { _label: "inv".to_string() }
    }

    /// f(x) 直传——期望 apply(|x| x*2+1, 5) == 11(接收者不参与计算)。
    pub fn apply(&self, f: Box<dyn Fn(i64) -> i64>, x: i64) -> i64 {
        f(x)
    }
}
