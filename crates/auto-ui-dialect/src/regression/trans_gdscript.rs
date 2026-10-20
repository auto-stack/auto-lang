//! PLAN-814 R3-03b：a2gd 转译回归——原 auto-lang-core `trans/gdscript.rs`
//! 的 `ui-dialect` 门控测试随迁（语料输入含 UI 语句，依赖文法表装配；
//! 语料目录 `crates/auto-lang/test/a2gd` 路径在本 crate 下同深解析）。
//! 语料驱动 helper 为 SDK 侧同名私有 helper 的等价拷贝。

use std::fs::read_to_string;
use std::path::PathBuf;

use auto_lang_core::error::AutoResult;
use auto_lang_core::parser::Parser;
use auto_lang_core::scope_manager::ScopeManager;
use auto_lang_core::trans::gdscript::GDScriptTrans;
use auto_lang_core::trans::{Sink, Trans};

#[cfg(test)]
mod tests {
    use super::*;

    /// PLAN-814 R3-03b：装配 UI 文法函数表（迁移前由 SDK `ui-dialect`
    /// feature 编译期提供；此处进程级注入单源构造器，语义等价）。
    fn setup() {
        let _ = auto_lang_core::set_ui_grammar(crate::ui_grammar());
    }

    fn test_a2gd(case: &str) -> AutoResult<()> {
        let last_segment = case.rsplit('/').next().unwrap_or(case);
        let parts: Vec<&str> = last_segment.split('_').collect();
        let name = parts[1..].join("_");
        let name = name.as_str();

        let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let src_path = format!("../auto-lang/test/a2gd/{}/{}.at", case, name);
        let src_path = d.join(src_path);
        let src = read_to_string(src_path.as_path())?;

        let _scope = ScopeManager::new();
        let mut parser = Parser::from(src.as_str());
        let ast = parser.parse()?;
        let mut sink = Sink::new(name.into());
        let mut trans = GDScriptTrans::new(name.into());
        trans.trans(ast, &mut sink)?;
        let gd_code = sink.done()?;

        let expected_path = format!("../auto-lang/test/a2gd/{}/{}.expected.gd", case, name);
        let expected_path = d.join(expected_path);
        let expected = read_to_string(expected_path.as_path())?;

        if gd_code != expected.as_bytes() {
            let gen_path = format!("../auto-lang/test/a2gd/{}/{}.wrong.gd", case, name);
            let gen_path = d.join(gen_path);
            std::fs::write(&gen_path, gd_code)?;
        }

        assert_eq!(String::from_utf8_lossy(gd_code), expected);
        Ok(())
    }

    /// Run a2gd on a dedicated large-stack thread (deep recursion in the
    /// transpiler hot path; mirrors test_a2r_deep pattern).
    fn test_a2gd_deep(case: &str) {
        let case = case.to_string();
        let child = std::thread::Builder::new()
            .stack_size(16 * 1024 * 1024)
            .spawn(move || test_a2gd(&case).map_err(|e| e.to_string()))
            .expect("failed to spawn deep-stack test thread");
        child
            .join()
            .expect("deep-stack test thread panicked")
            .unwrap();
    }

    // Plan 306: signal declarations inside a scene → `signal name(p: T)` in .gd.
    #[test]
    fn test_godot_signals() {
        setup();
        test_a2gd("17_godot_types/005_signal").unwrap();
    }

    // Plan 308: reverse-translated Godot demo scripts.
    #[test]
    fn test_godot_demo_instancing_ball_factory() {
        setup();
        test_a2gd("tscn/godot_demos/instancing/002_ball_factory").unwrap();
    }

    #[test]
    fn test_godot_demo_hexagonal_troll() {
        setup();
        test_a2gd("tscn/godot_demos/hexagonal_map/001_troll").unwrap();
    }

    #[test]
    fn test_godot_demo_kinematic_player() {
        setup();
        test_a2gd("tscn/godot_demos/kinematic_character/001_player").unwrap();
    }

    #[test]
    fn test_godot_demo_dodge_mob() {
        setup();
        test_a2gd("tscn/godot_demos/dodge_the_creeps/001_mob").unwrap();
    }

    #[test]
    fn test_godot_demo_dodge_player() {
        setup();
        // Deep-stack: 002_player drives deep recursion in the transpiler hot
        // path (see test_a2gd_deep). Stack-budget, not a regression.
        test_a2gd_deep("tscn/godot_demos/dodge_the_creeps/002_player");
    }
}
