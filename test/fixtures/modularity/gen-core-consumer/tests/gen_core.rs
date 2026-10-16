//! PLAN-791 V-01 — an EXTERNAL consumer using only `auto-gen-core`.
//!
//! Everything here must hold without auto-lang/auto-man/auto-vm anywhere in
//! the build (enforced structurally by this crate's manifest and by
//! `verify-deps.sh`). The fake renderer proves the core really writes files
//! and honors the guard/dry_run/failure contracts on its own.

use auto_gen_core::{
    generate, GenError, OutputConfig, RenderedTemplate, TemplateRenderer, TemplateSpec,
};
use std::path::{Path, PathBuf};

/// Fake renderer: maps template file stem -> owned text; stems in `fail`
/// fail with a render error (like a broken template).
struct FakeRenderer {
    contents: Vec<(String, String)>,
    fail: Vec<String>,
}

impl FakeRenderer {
    fn with_stem(mut self, stem: &str, content: &str) -> Self {
        self.contents.push((stem.to_string(), content.to_string()));
        self
    }

    fn failing(mut self, stem: &str) -> Self {
        self.fail.push(stem.to_string());
        self
    }

    fn lookup(&self, stem: &str) -> Option<&str> {
        self.contents
            .iter()
            .find(|(k, _)| k == stem)
            .map(|(_, v)| v.as_str())
    }
}

impl TemplateRenderer for FakeRenderer {
    fn render(&mut self, spec: &TemplateSpec) -> Result<RenderedTemplate, GenError> {
        let stem = spec
            .template_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("template")
            .to_string();
        if self.fail.contains(&stem) {
            return Err(GenError::Other(format!("fake render failure for {stem}")));
        }
        match self.lookup(&stem) {
            Some(content) => Ok(RenderedTemplate {
                template_name: stem.into(),
                content: content.to_string(),
            }),
            None => Err(GenError::TemplateLoadError {
                path: spec.template_path.clone(),
                reason: "unknown to fake renderer".to_string(),
            }),
        }
    }
}

fn renderer() -> FakeRenderer {
    FakeRenderer {
        contents: Vec::new(),
        fail: Vec::new(),
    }
}

fn spec(stem: &str) -> TemplateSpec {
    TemplateSpec {
        template_path: PathBuf::from(format!("{}.txt.at", stem)),
        output_name: None,
        rename: false,
    }
}

fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "gen_core_consumer_{}_{}_{}",
        env!("CARGO_PKG_NAME"),
        tag,
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn output_config(dir: &Path) -> OutputConfig {
    OutputConfig {
        output_dir: dir.to_path_buf(),
        dry_run: false,
        overwrite_guarded: false,
    }
}

/// V-01 main oracle: fake text REALLY reaches files through the public API.
#[test]
fn external_fake_renderer_writes_real_files() {
    let dir = scratch("writes");
    let mut r = renderer().with_stem("greeting.txt", "Hello, external world!\nsecond line");
    let report = generate(&[spec("greeting")], &output_config(&dir), &mut r);

    assert!(report.errors.is_empty(), "{:?}", report.errors.len());
    assert_eq!(report.files_generated, vec![dir.join("greeting.txt")]);
    let bytes = std::fs::read(dir.join("greeting.txt")).unwrap();
    assert_eq!(
        String::from_utf8(bytes).unwrap(),
        "Hello, external world!\nsecond line"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// Guard preservation is available to pure-core consumers too.
#[test]
fn external_guard_section_is_preserved() {
    let dir = scratch("guard");
    std::fs::write(
        dir.join("guarded.txt"),
        "old\n/// ---------- begin of guard: <u> ---\nHAND EDITED\n/// ---------- end of guard: ---\nold tail\n",
    )
    .unwrap();
    let mut r = renderer().with_stem(
        "guarded.txt",
        "new\n/// ---------- begin of guard: <u> ---\nTEMPLATE\n/// ---------- end of guard: ---\nnew tail\n",
    );
    let report = generate(&[spec("guarded")], &output_config(&dir), &mut r);
    assert!(report.errors.is_empty());
    let out = std::fs::read_to_string(dir.join("guarded.txt")).unwrap();
    assert!(out.contains("HAND EDITED"));
    assert!(!out.contains("TEMPLATE"));
    let _ = std::fs::remove_dir_all(&dir);
}

/// dry_run performs zero filesystem effects and failing templates never
/// touch existing outputs.
#[test]
fn external_dry_run_and_failure_backstops() {
    let dir = scratch("dry");
    let mut r = renderer().with_stem("greeting.txt", "text");
    let report = generate(
        &[spec("greeting")],
        &OutputConfig {
            output_dir: dir.clone(),
            dry_run: true,
            overwrite_guarded: false,
        },
        &mut r,
    );
    assert_eq!(report.files_generated.len(), 1);
    assert!(std::fs::read_dir(&dir).unwrap().count() == 0);

    let sentinel = b"untouched\n";
    std::fs::write(dir.join("broken.txt"), sentinel).unwrap();
    let mut r2 = renderer().failing("broken.txt");
    let report2 = generate(&[spec("broken")], &output_config(&dir), &mut r2);
    assert_eq!(report2.errors.len(), 1);
    assert_eq!(std::fs::read(dir.join("broken.txt")).unwrap(), sentinel);
    let _ = std::fs::remove_dir_all(&dir);
}

/// The legacy guard feed keeps its distinct byte semantics for pure-core
/// consumers (join without trailing newline).
#[test]
fn external_legacy_feed_byte_semantics() {
    let dir = scratch("legacy");
    let file = dir.join("existing.txt");
    std::fs::write(
        &file,
        "a\n/// ---------- begin of guard: <k> ---\nKEPT\n/// ---------- end of guard: ---\nb\n",
    )
    .unwrap();
    // AutoStr converts from &str; the consumer never needs to name auto_val
    // directly (the manifest depends on auto-gen-core ONLY).
    let input = "x\n/// ---------- begin of guard: <k> ---\nNEW\n/// ---------- end of guard: ---\ny\n";
    let fed = auto_gen_core::feed_guarded_legacy(&file, input.into());
    assert_eq!(
        fed,
        "x\n/// ---------- begin of guard: <k> ---\nKEPT\n/// ---------- end of guard: ---\ny"
    );
    let _ = std::fs::remove_dir_all(&dir);
}
