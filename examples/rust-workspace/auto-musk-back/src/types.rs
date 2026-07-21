use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ToolCall {
    pub tool: String,
    pub args: any,
    pub result: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RunResult {
    pub output: String,
    pub turns: i64,
    pub tool_calls: Vec<ToolCall>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Profession {
    pub name: String,
    pub model: String,
    pub temperature: f64,
    pub max_turns: i64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct WorkflowResult {
    pub steps: any,
    pub outputs: any,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct UserInfo {
    pub username: String,
    pub role: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct LoginResult {
    pub token: String,
    pub user: UserInfo,
}
