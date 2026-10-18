//! 工程场景类型（Plan 130）。
//! （PLAN-809 T-03 自 auto-man/src/pac.rs 迁入；语义逐字冻结。）

/// 工程场景类型 (Plan 130)
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Scene {
    /// Workspace 类型 - 包含多个子工程
    Workspace,
    /// UI 工程 - AURA 语法
    Ui,
    /// C 工程 - 原生 Auto/C 构建管线（pac.at `scene: "c"`）
    C,
    /// 未声明 scene 的旧工程 - 由工具推断（声明了 port 且无 render 视同 C）
    #[default]
    Default,
}

impl Scene {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "workspace" => Self::Workspace,
            "ui" => Self::Ui,
            "c" | "core" => Self::C,
            _ => Self::Default,
        }
    }
}

pub fn is_workspace(scene: &Scene) -> bool {
    scene == &Scene::Workspace
}

pub fn is_ui(scene: &Scene) -> bool {
    scene == &Scene::Ui
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scene_parsing() {
        assert!(matches!(Scene::from_str("c"), Scene::C));
        assert!(matches!(Scene::from_str("C"), Scene::C));
        assert!(matches!(Scene::from_str("core"), Scene::C));
        assert!(matches!(Scene::from_str("ui"), Scene::Ui));
        assert!(matches!(Scene::from_str("workspace"), Scene::Workspace));
        // absent / unrecognized → Default (legacy, inference applies)
        assert!(matches!(Scene::from_str(""), Scene::Default));
        assert!(matches!(Scene::from_str("typo"), Scene::Default));
    }
}
