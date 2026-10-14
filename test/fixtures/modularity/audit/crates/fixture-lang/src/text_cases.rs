//! 文本候选分类夹具（PLAN-777 V-03）：七处受管命中 + 一处别名盲区。
//! 本文件只服务于 modularity-audit 的扫描器自测，不参与任何真实编译。

/// doc 注释命中：crate::fixture_ui::in_doc
// 行注释命中：crate::fixture_ui::in_comment
/* 同行块注释命中：crate::fixture_ui::in_block_comment */
pub fn code_hit() {
    // 别名 use 语句本体是一次 code 命中；其后经别名的调用是已声明盲区
    use crate::fixture_ui as fui;
    let _ = fui::via_alias();
}

pub fn string_hit() -> &'static str {
    "字符串命中：crate::fixture_ui::in_string"
}

macro_rules! emit_ui_ref {
    () => {
        crate::fixture_ui::in_macro()
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn refs_in_test() {
        let _ = crate::fixture_ui::in_test();
    }
}
