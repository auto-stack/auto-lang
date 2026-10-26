use super::{Expr, Name, Type};
use auto_val::AutoStr;

#[derive(Debug, Clone)]
pub struct SceneDecl {
    pub name: Name,
    pub node_type: Name,
    pub props: Vec<SceneProp>,
    pub script: Option<AutoStr>,
    pub children: Vec<SceneNode>,
    pub connections: Vec<SceneConnection>,
    pub signals: Vec<SceneSignal>,
}

#[derive(Debug, Clone)]
pub struct SceneProp {
    pub name: Name,
    pub value: SceneValue,
}

#[derive(Debug, Clone)]
pub enum SceneValue {
    Expr(Expr),
    SubResource(SceneSubResource),
}

#[derive(Debug, Clone)]
pub struct SceneSubResource {
    pub res_type: Name,
    pub props: Vec<SceneProp>,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct SceneConnection {
    pub signal: AutoStr,
    pub from: AutoStr,
    pub to: AutoStr,
    pub method: AutoStr,
}

#[derive(Debug, Clone)]
pub struct SceneSignal {
    pub name: AutoStr,
    pub params: Vec<SceneSignalParam>,
}

#[derive(Debug, Clone)]
pub struct SceneSignalParam {
    pub name: Name,
    pub ty: Type,
}
