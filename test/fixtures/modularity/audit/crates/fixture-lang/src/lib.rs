//! fixture-lang：语言 core 占位（PLAN-777 modularity-audit 自测夹具）。
//! 本 workspace 永不编译；manifest 只为审计的声明边提取服务。

// text_cases 仅用于文本扫描器分类自测，不参与真实模块图。
#[allow(dead_code)]
mod text_cases;

pub const NAME: &str = "fixture-lang";
