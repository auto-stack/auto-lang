use crate::util::split_last;
use crate::{AutoError, AutoResult};
use auto_lang::interpreter::AutoInterpreter;
use auto_man_assets::{plan_template_family, GhsAssets, IarAssets, TemplateAssets};
use auto_val::AutoStr;
use auto_val::Value;
use rust_embed::{EmbeddedCompressedFile, EmbeddedFile, RustEmbed};
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::Path;

/// 模板脚手架兼容门面（PLAN-803）：资源唯一来源在
/// `crates/auto-man-assets`（`TemplateAssets`）。本类型保留旧公开面——
/// inherent `copy` 与 `RustEmbed` 的 get/iter 转发；不再 derive 第二份
/// 嵌入。Auto 求值与文件物化留在此上层（adapter）。
pub struct Templates;

impl Templates {
    // 复制模板到指定路径
    //
    // PLAN-803：枚举/族选择/路径 "name" 替换经
    // `auto_man_assets::plan_template_family`（旧臂冻结投影，顺序一致）；
    // 逐文件的求值/输出步骤保持旧臂原样——含 unwrap panic 语义与
    // 「先创建/截断文件再求值」的遗留顺序（行为改善另立计划，不在
    // 结构迁移中夹带）。
    pub fn copy(template: &str, path: &str) -> Result<(), AutoError> {
        let path = Path::new(path);
        let pac_name = path.file_name().unwrap().to_str().unwrap();
        for planned in plan_template_family(template, pac_name) {
            let sub_path = planned.output_rel;
            println!("Copying {}", sub_path);
            let (dir, _) = split_last(sub_path.as_ref(), '/');
            // 创建对应的文件夹
            create_dir_all(path.join(dir))?;
            // 创建文件
            let mut f = File::create(path.join(sub_path))?;
            // 模板原始字节（未求值）
            let code = String::from_utf8(planned.bytes.to_vec()).unwrap();
            // 替换模板中的 ${name} 为 pac_name
            let mut interp = AutoInterpreter::new();
            interp.set_global("name", Value::from(pac_name));
            let auto_code = interp.eval_template("", &code);
            let result = auto_code.unwrap();
            let code = result.repr();
            println!("code: {}", code);
            // 写入文件
            f.write_all(code.as_bytes())?;
        }
        Ok(())
    }
}

// inherent get/iter/compressed：旧 derive 除 trait impl 外还生成同名
// inherent 方法（无需 import trait 即可 `Templates::iter()` 调用）——
// 兼容门面逐形态保留。
impl Templates {
    pub fn get(file_path: &str) -> Option<EmbeddedFile> {
        TemplateAssets::get(file_path)
    }
    pub fn compressed(file_path: &str) -> Option<EmbeddedCompressedFile> {
        TemplateAssets::compressed(file_path)
    }
    pub fn iter() -> impl Iterator<Item = std::borrow::Cow<'static, str>> + 'static {
        TemplateAssets::iter()
    }
}

impl RustEmbed for Templates {
    fn get(file_path: &str) -> Option<EmbeddedFile> {
        TemplateAssets::get(file_path)
    }
    fn compressed(file_path: &str) -> Option<EmbeddedCompressedFile> {
        TemplateAssets::compressed(file_path)
    }
    fn iter() -> impl Iterator<Item = std::borrow::Cow<'static, str>> + 'static {
        TemplateAssets::iter()
    }
}

/// GHS 构建器模板兼容门面（PLAN-803）：字节唯一来源 `GhsAssets`，
/// 旧 inherent default/program/subproject 与 RustEmbed get/iter 保留。
pub struct GHSTemplate;

impl GHSTemplate {
    pub fn default() -> AutoResult<AutoStr> {
        let file = GhsAssets::get("default.gpj").unwrap();
        let str = String::from_utf8(file.data.as_ref().to_vec()).unwrap();
        Ok(str.into())
    }

    pub fn program() -> AutoResult<AutoStr> {
        let file = GhsAssets::get("program.gpj").unwrap();
        let str = String::from_utf8(file.data.as_ref().to_vec()).unwrap();
        Ok(str.into())
    }

    pub fn subproject() -> AutoResult<AutoStr> {
        let file = GhsAssets::get("subproject.gpj").unwrap();
        let str = String::from_utf8(file.data.as_ref().to_vec()).unwrap();
        Ok(str.into())
    }
}

impl GHSTemplate {
    pub fn get(file_path: &str) -> Option<EmbeddedFile> {
        GhsAssets::get(file_path)
    }
    pub fn compressed(file_path: &str) -> Option<EmbeddedCompressedFile> {
        GhsAssets::compressed(file_path)
    }
    pub fn iter() -> impl Iterator<Item = std::borrow::Cow<'static, str>> + 'static {
        GhsAssets::iter()
    }
}

impl RustEmbed for GHSTemplate {
    fn get(file_path: &str) -> Option<EmbeddedFile> {
        GhsAssets::get(file_path)
    }
    fn compressed(file_path: &str) -> Option<EmbeddedCompressedFile> {
        GhsAssets::compressed(file_path)
    }
    fn iter() -> impl Iterator<Item = std::borrow::Cow<'static, str>> + 'static {
        GhsAssets::iter()
    }
}

/// IAR 构建器模板兼容门面（PLAN-803）：字节唯一来源 `IarAssets`，
/// 旧 inherent eww/ewp/ewt/ewd 与 RustEmbed get/iter 保留。
pub struct IARTemplate;

impl IARTemplate {
    pub fn eww() -> Result<AutoStr, AutoError> {
        let file = IarAssets::get("iar.eww").unwrap();
        let str = String::from_utf8(file.data.as_ref().to_vec()).unwrap();
        Ok(AutoStr::from(str))
    }

    pub fn ewp() -> Result<AutoStr, AutoError> {
        let file = IarAssets::get("iar.ewp").unwrap();
        let str = String::from_utf8(file.data.as_ref().to_vec()).unwrap();
        Ok(AutoStr::from(str))
    }

    pub fn ewt() -> Result<AutoStr, AutoError> {
        let file = IarAssets::get("iar.ewt").unwrap();
        let str = String::from_utf8(file.data.as_ref().to_vec()).unwrap();
        Ok(AutoStr::from(str))
    }

    pub fn ewd() -> Result<AutoStr, AutoError> {
        let file = IarAssets::get("iar.ewd").unwrap();
        let str = String::from_utf8(file.data.as_ref().to_vec()).unwrap();
        Ok(AutoStr::from(str))
    }
}

impl IARTemplate {
    pub fn get(file_path: &str) -> Option<EmbeddedFile> {
        IarAssets::get(file_path)
    }
    pub fn compressed(file_path: &str) -> Option<EmbeddedCompressedFile> {
        IarAssets::compressed(file_path)
    }
    pub fn iter() -> impl Iterator<Item = std::borrow::Cow<'static, str>> + 'static {
        IarAssets::iter()
    }
}

impl RustEmbed for IARTemplate {
    fn get(file_path: &str) -> Option<EmbeddedFile> {
        IarAssets::get(file_path)
    }
    fn compressed(file_path: &str) -> Option<EmbeddedCompressedFile> {
        IarAssets::compressed(file_path)
    }
    fn iter() -> impl Iterator<Item = std::borrow::Cow<'static, str>> + 'static {
        IarAssets::iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_templates() {
        for template in Templates::iter() {
            println!("{}", template.as_ref());
        }
    }
}
