// Plan 152: SSE 类型定义
//
// Server-Sent Events (SSE) 类型定义

use serde::{Deserialize, Serialize};

/// SSE 事件
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SSEEvent {
    /// 事件 ID
    pub id: Option<String>,
    /// 事件类型
    pub event: Option<String>,
    /// 事件数据
    pub data: String,
    /// 重连延迟（毫秒）
    pub retry: Option<u32>,
}

impl SSEEvent {
    /// 创建新的 SSE 事件
    pub fn new() -> Self {
        Self {
            id: None,
            event: None,
            data: String::new(),
            retry: None,
        }
    }

    /// 设置事件 ID
    pub fn with_id(mut self, id: String) -> Self {
        self.id = Some(id);
        self
    }

    /// 设置事件类型
    pub fn with_event(mut self, event: String) -> Self {
        self.event = Some(event);
        self
    }

    /// 设置事件数据
    pub fn with_data(mut self, data: String) -> Self {
        self.data = data;
        self
    }

    /// 设置重连延迟
    pub fn with_retry(mut self, retry: u32) -> Self {
        self.retry = Some(retry);
        self
    }

    /// 检查是否为空事件
    pub fn is_empty(&self) -> bool {
        self.data.is_empty() && self.id.is_none() && self.event.is_none()
    }

    /// 检查是否为完成标记
    pub fn is_done(&self) -> bool {
        self.data.trim() == "[DONE]"
    }
}

impl Default for SSEEvent {
    fn default() -> Self {
        Self::new()
    }
}

/// SSE 解析错误
#[derive(Debug, thiserror::Error)]
pub enum SSEError {
    #[error("Invalid SSE format: {0}")]
    InvalidFormat(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
}

/// SSE 解析结果
pub type SSEResult<T> = Result<T, SSEError>;
