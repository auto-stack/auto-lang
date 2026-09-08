//! plan-430 C2 端到端夹具 crate:分类器各规则的可调用面。
//! PLAN-592 追加:DROPS 计数器(drop 生命周期观测面 + 自由函数 ()→u64 冒烟)。

use std::sync::atomic::{AtomicUsize, Ordering};

static DROPS: AtomicUsize = AtomicUsize::new(0);

/// 已析构的对象数(Counter/Config/Point 合计)——生命周期断言的确定性观测点。
/// 注:i64 返回——212 自由函数 wrapper 的 u64↔i64 槽无收窄转换(DIV-DEP-6),
/// u64 返回特征化由 016 的 Num.u64_max 方法路径覆盖。
pub fn drop_count() -> i64 {
    DROPS.load(Ordering::SeqCst) as i64
}

pub struct Counter {
    count: i64,
    label: String,
}

impl Counter {
    /// 静态构造器 + 按值 String 参数(StrOwned → TakeStr)
    pub fn new(label: String) -> Counter {
        Counter { count: 0, label }
    }

    /// &mut self → void
    pub fn increment(&mut self) {
        self.count += 1;
    }

    /// &self → i64(规则 6:宽整型 i64 槽)
    pub fn value(&self) -> i64 {
        self.count
    }

    /// &self → String
    pub fn label(&self) -> String {
        self.label.clone()
    }

    /// &mut self + String 参数 → void
    pub fn set_label(&mut self, label: String) {
        self.label = label;
    }

    /// &mut self + i64 参数 → i64
    pub fn add(&mut self, delta: i64) -> i64 {
        self.count += delta;
        self.count
    }

    /// &self → 不透明返回(压新句柄)
    pub fn clone_reset(&self) -> Counter {
        Counter {
            count: self.count,
            label: self.label.clone(),
        }
    }

    /// 关联函数(无 self)→ String
    pub fn version() -> String {
        "1.0.0".to_string()
    }

    /// Option 返回:v1 分类策略应跳过(unwrap policy pending)
    pub fn maybe(&self) -> Option<i64> {
        if self.count > 0 {
            Some(self.count)
        } else {
            None
        }
    }
}

impl Drop for Counter {
    fn drop(&mut self) {
        DROPS.fetch_add(1, Ordering::SeqCst);
    }
}

/// 自由函数:仅进 manifest 元数据(D2),代码生成走 plan-212 syn 路径
pub fn describe(c: &Counter) -> String {
    format!("{}={}", c.label, c.count)
}

/// builder 链式(&mut self 返回 &mut Self → ChainInPlace,压回原句柄)
/// 与按值 self(self → Self,消耗接收者)与 opaque 参数的覆盖面。
pub struct Config {
    verbose: bool,
    level: i64,
}

impl Config {
    pub fn new() -> Config {
        Config { verbose: false, level: 0 }
    }

    /// ChainInPlace:原地修改,返回自身引用
    pub fn verbose(&mut self, on: bool) -> &mut Config {
        self.verbose = on;
        self
    }

    /// ChainInPlace + i64 参数
    pub fn level(&mut self, n: i64) -> &mut Config {
        self.level = n;
        self
    }

    pub fn is_verbose(&self) -> bool {
        self.verbose
    }

    pub fn level_value(&self) -> i64 {
        self.level
    }

    /// 按值 self 返回 Self:v1 跳过(消耗语义 × chain 别名的句柄失效策略待例外层)
    #[allow(dead_code)]
    pub fn bump(self) -> Config {
        Config { level: self.level + 1, verbose: self.verbose }
    }

    /// opaque 参数:接收者句柄 + 另一 Config 句柄
    pub fn merge(&self, other: &Config) -> Config {
        Config { level: self.level.max(other.level), verbose: self.verbose || other.verbose }
    }

    /// Result 返回:unwrap_ok 策略(Err 经 cdylib 错误通道 → VMError)
    pub fn parse(text: String) -> Result<Config, String> {
        let n = text
            .trim()
            .parse::<i64>()
            .map_err(|e| format!("invalid level `{text}`: {e}"))?;
        Ok(Config { verbose: false, level: n })
    }
}

impl Drop for Config {
    fn drop(&mut self) {
        DROPS.fetch_add(1, Ordering::SeqCst);
    }
}

/// 公共字段(getter 合成)与 Display(to_string 合成)的覆盖面
pub struct Point {
    pub x: i64,
    pub y: i64,
    pub tag: String,
}

impl Point {
    pub fn new(x: i64, y: i64, tag: String) -> Point {
        Point { x, y, tag }
    }
}

impl std::fmt::Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}) {}", self.x, self.y, self.tag)
    }
}

impl Drop for Point {
    fn drop(&mut self) {
        DROPS.fetch_add(1, Ordering::SeqCst);
    }
}
