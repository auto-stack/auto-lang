//! 双 source 负控宿主（见 Cargo.toml）——预期永远构建失败：依赖解析层
//! 应拒绝同名 `auto-ui-dialect` 双路径。main 体不参与判定。
fn main() {
    println!("plan831-compile-probe-dual: 本 crate 预期在依赖解析层失败（同名包双 source）");
}
