//! PLAN-591 T8: 孪生 fixture(对抗①)——与 autolang_shapes 的 Messy 同类型名、
//! 同方法签名集,**异字段声明序** + 异哨兵值。若布局信息混淆(陈旧 pack 复用/
//! layouts 串键),offset 直读即返回对方真值——真值断言当场暴露。

pub struct Messy {
    pub flag: bool,
    pub total: i64,
    pub b: u8,
    pub tag: String,
    pub a: u8,
}

impl Messy {
    pub fn new(total: i64, tag: &str, flag: bool) -> Self {
        Self {
            flag,
            total,
            b: 13,
            tag: tag.to_string(),
            a: 9,
        }
    }

    pub fn total_snapshot(&self) -> i64 {
        self.total
    }
}
