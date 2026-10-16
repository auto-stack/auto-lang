//! auto-man-assets — Man 模板/构建器资源单一来源（PLAN-803）。
//!
//! 三组资源自 `crates/auto-man/assets/{templates, builders/ghs,
//! builders/iar}` 经 git mv 逐字迁入（字节/键/枚举顺序保持）。本 crate
//! 无语言引擎：不依赖 auto-lang/auto-val/auto-man/auto-gen/VM/UI，纯资源
//! 消费者只需 rust-embed 即可枚举读取全部资源并按模板族+包名生成
//! 「源资源键 → 相对输出路径」的纯计划（[`plan_template_family`]）。
//!
//! 纯计划无输出 IO：不创建目录、不渲染、不写文件；模板内容中的
//! `${name}` 保持原始字节（Auto 求值与文件物化留在 auto-man 的
//! asset.rs 兼容门面上层）。**构建模式边界**（F803-R1-03）：本 crate
//! 随 workspace 只启用 rust-embed `compression`（无 `debug-embed`），
//! 默认调试态 `iter()/get()` 为动态模式——枚举/读取宏展开期记录的源码
//! 资源目录（有读盘，目录缺失时枚举静默为空）；发行态（或显式
//! `debug-embed`）字节内嵌二进制，与源码目录无关。可重复反控：
//! `test/fixtures/modularity/man-assets-consumer/mode-boundary-probe.py`。
//! 依赖闭包由
//! `test/fixtures/modularity/man-assets-consumer`（PLAN-803 V01）钉住。

use rust_embed::Embed;
use std::borrow::Cow;

/// 模板族资源根：app/capp/clib/lib/libdemo 五族（13 文件）。
/// 键为相对本根的 `/` 分隔路径（如 `libdemo/libs/name/name.c`）。
#[derive(Embed)]
#[folder = "assets/templates"]
pub struct TemplateAssets;

/// GHS 构建器工程模板（default/program/subproject .gpj，3 文件）。
#[derive(Embed)]
#[folder = "assets/builders/ghs"]
pub struct GhsAssets;

/// IAR 构建器工程模板（iar.eww/ewp/ewt/ewd，4 文件）。
#[derive(Embed)]
#[folder = "assets/builders/iar"]
pub struct IarAssets;

/// 一条计划输出：嵌入源键 + 旧臂字面 `"name"`→包名替换后的相对输出
/// 路径 + 未求值原始字节。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedFile {
    /// 相对 `assets/templates` 根的嵌入键（如 `app/pac.at`）。
    pub source_key: String,
    /// 旧臂路径规划结果：**去掉族前缀后的余段**对子串 `"name"` 的字面
    /// 全量替换（包名 `mylib` 时 `name.at`→`mylib.at`、
    /// `libdemo/apps/name_demo/name_demo.c` 的余段
    /// `apps/name_demo/name_demo.c`→`apps/mylib_demo/mylib_demo.c`）。
    pub output_rel: String,
    /// 模板原始字节（`${name}` 未渲染——求值属 auto-man 上层）。
    pub bytes: Cow<'static, [u8]>,
}

/// 旧臂首段切分语义（auto-man `util::split_first` 的逐字镜像）：
/// `splitn(2, '/')` 取两段；无分隔符时第二个 `unwrap` 与旧臂同样 panic
/// （嵌入键恒含 `/`，实际不可达；镜像保留原语义而非悄悄改写）。
fn split_first_segment(path: &str) -> (&str, &str) {
    let mut splitter = path.splitn(2, '/');
    let first = splitter.next().unwrap();
    let second = splitter.next().unwrap();
    (first, second)
}

/// 按模板族规划输出文件清单——旧臂 `Templates::copy` 的枚举/选择/路径
/// 替换冻结投影：
///
/// - 以 `TemplateAssets::iter()` 顺序枚举（rust-embed walkdir
///   `sort_by_file_name`，与旧臂同一机制同一布局，顺序保持）；
/// - 首段等于 `template` 的键入选（旧臂 `folder == template`）；
/// - 输出相对路径 = 余段 `sub_path.replace("name", pac_name)` 逐字同臂；
/// - 未知族返回空 vec（旧臂零匹配、零副作用）。
///
/// 接收包名字符串，不读 pac、不执行 Auto、无输出 IO（不创建目录、
/// 不渲染、不写文件）。注意：默认调试态下 rust-embed 动态枚举会读
/// 源码资源目录（见模块级「构建模式边界」说明）；发行态/`debug-embed`
/// 则读内嵌字节。
pub fn plan_template_family(template: &str, pac_name: &str) -> Vec<PlannedFile> {
    let mut out = Vec::new();
    for key in TemplateAssets::iter() {
        let key = key.as_ref();
        let (folder, sub_path) = split_first_segment(key);
        if folder == template {
            out.push(PlannedFile {
                source_key: key.to_string(),
                output_rel: sub_path.replace("name", pac_name),
                bytes: TemplateAssets::get(key).unwrap().data,
            });
        }
    }
    out
}

#[cfg(test)]
mod tests;
