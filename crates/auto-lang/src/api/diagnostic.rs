//! PLAN-734 T-02：结构化生成诊断。
//!
//! 生成期失败（解析/类型/转译/委派/能力）经 [`ApiDiagnostic`] 携带定位与
//! 行动指引向上传播——不是字符串注释或空返回（决策 D4；SD-04）。
//! v1 定位粒度 = 文件 + 端点 fn 名 + 阶段（AST span 管线另案——决策 D1）。

use std::fmt;

/// 诊断阶段（生成管线分节）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticStage {
    /// api.at 解析（strict full-parse）。
    Parse,
    /// 类型解析/契约分类。
    Type,
    /// 业务体 a2r 转译。
    Transpile,
    /// 实现来源解析（委派缺失/实现不可用）。
    Implementation,
    /// 传输能力（不支持面：File/Upload 经 IPC 等）。
    Capability,
    /// 产物写盘/发布。
    Publish,
}

impl DiagnosticStage {
    pub fn as_str(self) -> &'static str {
        match self {
            DiagnosticStage::Parse => "parse",
            DiagnosticStage::Type => "type",
            DiagnosticStage::Transpile => "transpile",
            DiagnosticStage::Implementation => "implementation",
            DiagnosticStage::Capability => "capability",
            DiagnosticStage::Publish => "publish",
        }
    }
}

/// 一条结构化生成诊断。
#[derive(Debug, Clone, PartialEq)]
pub struct ApiDiagnostic {
    /// 涉事端点 fn 名（模块级问题为 None）。
    pub endpoint: Option<String>,
    /// 源文件（相对项目根；v1 = 契约文件名）。
    pub file: Option<String>,
    pub stage: DiagnosticStage,
    /// 稳定原因码（机器可读：`unsupported_type` / `transpile_failed` / …）。
    pub reason: String,
    /// 人读详情（含涉事类型串/位置提示）。
    pub detail: String,
}

impl ApiDiagnostic {
    pub fn new(
        stage: DiagnosticStage,
        reason: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
        ApiDiagnostic {
            endpoint: None,
            file: None,
            stage,
            reason: reason.into(),
            detail: detail.into(),
        }
    }

    pub fn at_endpoint(mut self, fn_name: &str) -> Self {
        self.endpoint = Some(fn_name.to_string());
        self
    }

    pub fn in_file(mut self, file: &str) -> Self {
        self.file = Some(file.to_string());
        self
    }
}

impl fmt::Display for ApiDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[api:{}{}] {}{}",
            self.stage.as_str(),
            self.endpoint
                .as_deref()
                .map(|e| format!(":{e}"))
                .unwrap_or_default(),
            self.reason,
            if self.detail.is_empty() {
                String::new()
            } else {
                format!(": {}", self.detail)
            }
        )
    }
}

impl std::error::Error for ApiDiagnostic {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan734_diagnostic_display_shape() {
        let d = ApiDiagnostic::new(
            DiagnosticStage::Transpile,
            "transpile_failed",
            "branch return not supported",
        )
        .at_endpoint("upload_file")
        .in_file("src/back/api.at");
        assert_eq!(
            d.to_string(),
            "[api:transpile:upload_file] transpile_failed: branch return not supported"
        );
        assert!(d.to_string().contains("upload_file"));
    }
}
