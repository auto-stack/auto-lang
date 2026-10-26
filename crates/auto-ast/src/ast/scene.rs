use super::{Expr, Name, Type};
use auto_val::AutoStr;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SceneDecl {
    pub name: Name,
    pub node_type: Name,
    pub props: Vec<SceneProp>,
    pub script: Option<AutoStr>,
    pub children: Vec<SceneNode>,
    pub connections: Vec<SceneConnection>,
    pub signals: Vec<SceneSignal>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SceneProp {
    pub name: Name,
    pub value: SceneValue,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum SceneValue {
    Expr(Expr),
    SubResource(SceneSubResource),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SceneSubResource {
    pub res_type: Name,
    pub props: Vec<SceneProp>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum SceneNode {
    Node {
        node_type: Name,
        name: Option<AutoStr>,
        props: Vec<SceneProp>,
        children: Vec<SceneNode>,
    },
    Instance {
        name: AutoStr,
        path: AutoStr,
    },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SceneConnection {
    pub signal: AutoStr,
    pub from: AutoStr,
    pub to: AutoStr,
    pub method: AutoStr,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SceneSignal {
    pub name: AutoStr,
    pub params: Vec<SceneSignalParam>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SceneSignalParam {
    pub name: Name,
    pub ty: Type,
}
