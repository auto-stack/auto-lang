//! PLAN-814 R3-03b：a2tscn 转译回归——原 auto-lang-core `trans/tscn.rs`
//! 的 `ui-dialect` 门控测试随迁（语料输入含 UI 语句，依赖文法表装配；
//! 语料目录 `crates/auto-lang/test/a2gd/tscn` 路径在本 crate 下同深解析）。
//! 语料驱动 helper 为 SDK 侧同名私有 helper 的等价拷贝（SDK 侧服务
//! 非门控语料测试，保留原份）。

use std::fs::read_to_string;
use std::path::PathBuf;

use auto_lang_core::parser::Parser;
use auto_lang_core::trans::gdscript::GDScriptTrans;
use auto_lang_core::trans::tscn::generate_from_ast;
use auto_lang_core::trans::{Sink, Trans};

#[cfg(test)]
mod tests {
    use super::*;

    /// PLAN-814 R3-03b：装配 UI 文法函数表（迁移前由 SDK `ui-dialect`
    /// feature 编译期提供；此处进程级注入单源构造器，语义等价）。
    fn setup() {
        let _ = auto_lang_core::set_ui_grammar(crate::ui_grammar());
    }

    /// Parse `test/a2gd/tscn/{case}/{name}.at`, generate `.tscn`, and compare
    /// against `{case}/{name}.expected.tscn`. Reuses the a2gd test layout.
    fn test_a2tscn(case: &str) -> Result<(), Box<dyn std::error::Error>> {
        let last_segment = case.rsplit('/').next().unwrap_or(case);
        let parts: Vec<&str> = last_segment.split('_').collect();
        let name = parts[1..].join("_");

        let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let src_path = d.join(format!("../auto-lang/test/a2gd/tscn/{}/{}.at", case, name));
        let src = read_to_string(&src_path)?;

        let mut parser = Parser::from(src.as_str());
        let ast = parser.parse()?;
        let generated = generate_from_ast(&ast).ok_or("no SceneDecl found in input")?;

        let expected_path = d.join(format!(
            "../auto-lang/test/a2gd/tscn/{}/{}.expected.tscn",
            case, name
        ));
        let wrong_path = d.join(format!(
            "../auto-lang/test/a2gd/tscn/{}/{}.wrong.tscn",
            case, name
        ));

        let expected = match read_to_string(&expected_path) {
            Ok(content) => content,
            Err(_) => {
                // Bootstrap: expected file doesn't exist yet. Write the generated
                // output to .wrong.tscn for review, then copy to .expected.tscn.
                std::fs::write(&wrong_path, &generated)?;
                panic!(
                    "expected file missing for {} — wrote generated output to {} for review",
                    case,
                    wrong_path.display()
                );
            }
        };

        if generated != expected {
            std::fs::write(&wrong_path, &generated)?;
            panic!(
                "tscn mismatch for {}\n--- expected ---\n{}\n--- generated ---\n{}\n(wrote {})",
                case,
                expected,
                generated,
                wrong_path.display()
            );
        }
        // Touch the sink import so it is not flagged unused across configurations.
        let _ = Sink::new("tscn".into());
        Ok(())
    }

    /// Run a2tscn on a dedicated large-stack thread (deep scene-tree recursion
    /// in the demo files tips the 2 MB libtest stack once the AST grew — Plan
    /// 395 widened Expr/Call frames; mirrors test_a2gd_deep).
    fn test_a2tscn_deep(case: &str) {
        let case = case.to_string();
        let child = std::thread::Builder::new()
            .stack_size(16 * 1024 * 1024)
            .spawn(move || test_a2tscn(&case).map_err(|e| e.to_string()))
            .expect("failed to spawn deep-stack test thread");
        child
            .join()
            .expect("deep-stack test thread panicked")
            .unwrap();
    }

    #[test]
    fn test_tscn_001_hello() {
        setup();
        test_a2tscn("001_hello").unwrap();
    }

    #[test]
    fn test_tscn_002_player() {
        setup();
        test_a2tscn("002_player").unwrap();
    }

    #[test]
    fn test_tscn_003_timers() {
        setup();
        test_a2tscn("003_timers").unwrap();
    }

    #[test]
    fn test_tscn_004_nested() {
        setup();
        test_a2tscn("004_nested").unwrap();
    }

    #[test]
    fn test_tscn_005_subresource() {
        setup();
        test_a2tscn("005_subresource").unwrap();
    }

    // Plan 308: reverse-translated Godot demo scenes.
    #[test]
    fn test_godot_demo_instancing_ball_scene() {
        setup();
        test_a2tscn("godot_demos/instancing/001_ball").unwrap();
    }

    #[test]
    fn test_godot_demo_hexagonal_troll_scene() {
        setup();
        test_a2tscn("godot_demos/hexagonal_map/001_troll").unwrap();
    }

    #[test]
    fn test_godot_demo_kinematic_player_scene() {
        setup();
        test_a2tscn("godot_demos/kinematic_character/001_player").unwrap();
    }

    #[test]
    fn test_godot_demo_dodge_mob_scene() {
        setup();
        test_a2tscn("godot_demos/dodge_the_creeps/001_mob").unwrap();
    }

    #[test]
    fn test_godot_demo_dodge_player_scene() {
        setup();
        // Deep-stack: deep scene-tree recursion (see test_a2tscn_deep).
        test_a2tscn_deep("godot_demos/dodge_the_creeps/002_player");
    }

    /// Plan 306 Phase 2b: one .at file carries both a `scene` (→ .tscn) and
    /// functions (→ .gd). The .tscn comes from the scene; the GDScript pass
    /// must skip the SceneDecl without erroring and keep the functions.
    #[test]
    fn test_tscn_006_combined_with_gd() {
        setup();
        let src = r#"
scene Counter : Control {
    script = "counter.gd"
    node Label "Count" { text = "0" }
}

fn increment(n int) int { n + 1 }
"#;
        let mut parser = Parser::from(src);
        let ast = parser.parse().unwrap();

        // .tscn side: scene present with its script reference.
        let tscn = generate_from_ast(&ast).expect("scene present");
        assert!(
            tscn.contains(r#"[node name="Counter" type="Control"]"#),
            "tscn root node: {}",
            tscn
        );
        assert!(
            tscn.contains(r#"script = ExtResource("1")"#),
            "tscn script ref: {}",
            tscn
        );

        // .gd side: GDScriptTrans consumes the moved AST and skips SceneDecl.
        let mut sink = Sink::new("counter".into());
        let mut trans = GDScriptTrans::new("counter".into());
        trans
            .trans(ast, &mut sink)
            .expect("gd transpile must skip SceneDecl");
        let gd = String::from_utf8_lossy(sink.done().unwrap()).into_owned();
        assert!(gd.contains("func increment"), "gd keeps functions: {}", gd);
        assert!(
            !gd.to_lowercase().contains("scene"),
            "gd must not leak the scene declaration: {}",
            gd
        );
        // The script extends the scene's root node type, not the Node default.
        assert!(
            gd.contains("extends Control"),
            "gd extends scene root type: {}",
            gd
        );
    }
}
