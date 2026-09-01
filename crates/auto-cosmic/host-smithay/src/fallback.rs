//! 非 Linux stub：宿主合成循环为 Linux-only（Plan 509 路线 B）。

/// 恒定错误——cfg 门控保证 Windows 只编译本 stub，不引 smithay。
pub fn run_host(
    _frame_png: Option<&std::path::Path>,
    _max_frames: Option<u64>,
) -> Result<(), String> {
    Err("auto-smithay-host is Linux-only (Plan 509 route B); Windows builds keep a stub by cfg-gating".into())
}
