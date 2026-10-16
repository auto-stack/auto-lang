//! Generation orchestration core (PLAN-791): output naming, guard
//! preservation, dry-run/overwrite semantics, per-template error collection
//! and the single-run report. The core never loads data, parses templates or
//! holds an interpreter — adapters hand it owned rendered text through the
//! [`TemplateRenderer`] callback and keep every language-specific concern
//! (data loading, library paths, template parsing, error mapping) on their
//! side of the boundary.

use crate::error::GenError;
use crate::guard::GuardProcessor;
use auto_val::AutoStr;
use std::path::PathBuf;
use std::time::Duration;

/// Output-side configuration the core needs. Language-side settings (for
/// example the Auto f-string note) stay in the adapter's config.
#[derive(Debug, Clone)]
pub struct OutputConfig {
    pub output_dir: PathBuf,
    pub dry_run: bool,
    pub overwrite_guarded: bool,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from("."),
            dry_run: false,
            overwrite_guarded: false,
        }
    }
}

/// Specification for a single template. Shared verbatim with the historical
/// `auto_gen::TemplateSpec` (single definition; auto-gen re-exports it).
/// `rename` is carried for compatibility and is not consulted by the core,
/// matching the pre-extraction output naming exactly.
pub struct TemplateSpec {
    pub template_path: PathBuf,
    pub output_name: Option<AutoStr>,
    pub rename: bool,
}

/// One prepared template: the renderer's owned text plus the template name
/// used for default output naming.
pub struct RenderedTemplate {
    pub template_name: AutoStr,
    pub content: String,
}

/// Boundary the adapter implements to turn a template spec into owned text.
/// The core calls it once per template inside
/// [`generate`]; load and render failures surface as ordinary per-template
/// errors (batch semantics are unchanged from the pre-extraction generator).
pub trait TemplateRenderer {
    fn render(&mut self, spec: &TemplateSpec) -> Result<RenderedTemplate, GenError>;
}

/// Report from a generation task (single definition; auto-gen re-exports it).
pub struct GenReport {
    pub files_generated: Vec<PathBuf>,
    pub errors: Vec<GenError>,
    pub duration: Duration,
}

/// Run the output orchestration for `templates`.
///
/// Per-template errors are collected into the report and the batch continues;
/// a top-level `Err` stage (data/library loading) belongs to the adapter and
/// happens BEFORE this call, preserving the historical phase boundary.
/// Failure safety: an output file is only touched after its render and guard
/// merge succeeded, and `dry_run` performs zero filesystem effects.
pub fn generate(
    templates: &[TemplateSpec],
    config: &OutputConfig,
    renderer: &mut dyn TemplateRenderer,
) -> GenReport {
    let start = std::time::Instant::now();
    let guard_processor = GuardProcessor::new();

    let mut files_generated = Vec::new();
    let mut errors = Vec::new();

    for template_spec in templates {
        match generate_one(template_spec, config, renderer, &guard_processor) {
            Ok(output_path) => files_generated.push(output_path),
            Err(e) => errors.push(e),
        }
    }

    GenReport {
        files_generated,
        errors,
        duration: start.elapsed(),
    }
}

fn generate_one(
    template_spec: &TemplateSpec,
    config: &OutputConfig,
    renderer: &mut dyn TemplateRenderer,
    guard_processor: &GuardProcessor,
) -> Result<PathBuf, GenError> {
    // Load + render happen adapter-side; failures abort THIS template only.
    let rendered = renderer.render(template_spec)?;

    // Determine output path
    let output_path = resolve_output_path(template_spec, &rendered.template_name, config);

    // Process guard blocks if file exists
    let final_output = if output_path.exists() && !config.overwrite_guarded {
        let existing = std::fs::read_to_string(&output_path)?;
        guard_processor.merge(&existing, &rendered.content)?
    } else {
        rendered.content
    };

    // Write output
    if !config.dry_run {
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&output_path, final_output.as_bytes())?;
    }

    Ok(output_path)
}

fn resolve_output_path(
    template_spec: &TemplateSpec,
    template_name: &AutoStr,
    config: &OutputConfig,
) -> PathBuf {
    let output_name = if let Some(name) = &template_spec.output_name {
        name.to_string()
    } else {
        // Use template name, remove trailing .at if present
        let name = template_name.to_string();
        if name.ends_with(".at") {
            name[..name.len() - 3].to_string()
        } else {
            name
        }
    };

    config.output_dir.join(&output_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Fake renderer mapping template path stem -> (name, content); paths
    /// absent from the map fail with a render error like a broken template.
    struct FakeRenderer {
        outputs: std::collections::HashMap<String, String>,
        fail: Vec<String>,
    }

    impl FakeRenderer {
        fn new() -> Self {
            Self {
                outputs: std::collections::HashMap::new(),
                fail: Vec::new(),
            }
        }

        fn with(mut self, stem: &str, content: &str) -> Self {
            self.outputs.insert(stem.to_string(), content.to_string());
            self
        }

        fn failing(mut self, stem: &str) -> Self {
            self.fail.push(stem.to_string());
            self
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
                return Err(GenError::TemplateSyntaxError {
                    location: crate::error::SourceLocation::new(
                        PathBuf::from(format!("template:{}", stem)),
                        0,
                        0,
                    ),
                    message: "fake render failure".to_string(),
                });
            }
            match self.outputs.get(&stem) {
                Some(content) => Ok(RenderedTemplate {
                    template_name: stem.into(),
                    content: content.clone(),
                }),
                None => Err(GenError::TemplateLoadError {
                    path: spec.template_path.clone(),
                    reason: "not in fake map".to_string(),
                }),
            }
        }
    }

    fn spec(stem: &str) -> TemplateSpec {
        TemplateSpec {
            template_path: PathBuf::from(format!("{}.txt.at", stem)),
            output_name: None,
            rename: false,
        }
    }

    fn tempfile_dir() -> std::path::PathBuf {
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("gen_core_orch_{}_{}", std::process::id(), n));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn fake_text_reaches_file_with_default_naming() {
        let dir = tempfile_dir();
        let report = generate(
            &[spec("hello")],
            &OutputConfig {
                output_dir: dir.clone(),
                ..Default::default()
            },
            &mut FakeRenderer::new().with("hello.txt", "line one\nline two"),
        );
        assert!(report.errors.is_empty());
        assert_eq!(report.files_generated, vec![dir.join("hello.txt")]);
        let written = std::fs::read_to_string(dir.join("hello.txt")).unwrap();
        assert_eq!(written, "line one\nline two");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn render_failure_keeps_existing_bytes_and_batch_continues() {
        let dir = tempfile_dir();
        std::fs::write(dir.join("good.txt"), b"").unwrap();
        let sentinel = b"precious\n".to_vec();
        std::fs::write(dir.join("bad.txt"), &sentinel).unwrap();

        let mut renderer = FakeRenderer::new()
            .with("good.txt", "fresh")
            .failing("bad.txt");
        let report = generate(
            &[spec("good"), spec("bad")],
            &OutputConfig {
                output_dir: dir.clone(),
                ..Default::default()
            },
            &mut renderer,
        );
        assert_eq!(report.files_generated, vec![dir.join("good.txt")]);
        assert_eq!(report.errors.len(), 1);
        assert!(format!("{}", report.errors[0]).contains("template:bad.txt"));
        // good.txt pre-exists (even empty), so the guard-merge path applies and
        // re-adds the trailing newline — old GuardProcessor::merge semantics.
        assert_eq!(
            std::fs::read(dir.join("good.txt")).unwrap(),
            b"fresh
"
        );
        assert_eq!(std::fs::read(dir.join("bad.txt")).unwrap(), sentinel);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn dry_run_has_zero_filesystem_effects() {
        let dir = tempfile_dir();
        let report = generate(
            &[spec("hello")],
            &OutputConfig {
                output_dir: dir.clone(),
                dry_run: true,
                ..Default::default()
            },
            &mut FakeRenderer::new().with("hello.txt", "text"),
        );
        assert_eq!(report.files_generated.len(), 1);
        assert!(report.errors.is_empty());
        assert!(std::fs::read_dir(&dir).unwrap().count() == 0);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn guard_merge_preserves_existing_section() {
        let dir = tempfile_dir();
        std::fs::write(
            dir.join("guarded.txt"),
            "stale\n/// ---------- begin of guard: <u> ---\nHAND EDITED\n/// ---------- end of guard: ---\nstale tail\n",
        )
        .unwrap();
        let report = generate(
            &[spec("guarded")],
            &OutputConfig {
                output_dir: dir.clone(),
                ..Default::default()
            },
            &mut FakeRenderer::new().with(
                "guarded.txt",
                "fresh\n/// ---------- begin of guard: <u> ---\nTEMPLATE\n/// ---------- end of guard: ---\nfresh tail\n",
            ),
        );
        assert!(report.errors.is_empty(), "{:?}", report.errors);
        let got = std::fs::read_to_string(dir.join("guarded.txt")).unwrap();
        assert_eq!(
            got,
            "fresh\n/// ---------- begin of guard: <u> ---\nHAND EDITED\n/// ---------- end of guard: ---\nfresh tail\n"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn overwrite_guarded_replaces_section() {
        let dir = tempfile_dir();
        std::fs::write(
            dir.join("guarded.txt"),
            "stale\n/// ---------- begin of guard: <u> ---\nHAND EDITED\n/// ---------- end of guard: ---\nstale tail\n",
        )
        .unwrap();
        let report = generate(
            &[spec("guarded")],
            &OutputConfig {
                output_dir: dir.clone(),
                overwrite_guarded: true,
                ..Default::default()
            },
            &mut FakeRenderer::new().with(
                "guarded.txt",
                "fresh\n/// ---------- begin of guard: <u> ---\nTEMPLATE\n/// ---------- end of guard: ---\nfresh tail\n",
            ),
        );
        assert!(report.errors.is_empty());
        let got = std::fs::read_to_string(dir.join("guarded.txt")).unwrap();
        assert!(got.contains("TEMPLATE"));
        assert!(!got.contains("HAND EDITED"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn nested_output_name_creates_dirs_only_on_success() {
        let dir = tempfile_dir();
        let nested = TemplateSpec {
            template_path: PathBuf::from("deep.txt.at"),
            output_name: Some("a/b/c.txt".into()),
            rename: false,
        };
        generate(
            &[nested],
            &OutputConfig {
                output_dir: dir.clone(),
                ..Default::default()
            },
            &mut FakeRenderer::new().with("deep.txt", "nested ok"),
        );
        assert_eq!(
            std::fs::read_to_string(dir.join("a/b/c.txt")).unwrap(),
            "nested ok"
        );

        let dir2 = tempfile_dir();
        let nested_fail = TemplateSpec {
            template_path: PathBuf::from("boom.txt.at"),
            output_name: Some("a/b/c.txt".into()),
            rename: false,
        };
        let report = generate(
            &[nested_fail],
            &OutputConfig {
                output_dir: dir2.clone(),
                ..Default::default()
            },
            &mut FakeRenderer::new().failing("boom.txt"),
        );
        assert_eq!(report.errors.len(), 1);
        assert!(!dir2.join("a").exists());
        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::remove_dir_all(&dir2);
    }

    #[test]
    fn double_at_suffix_is_stripped_once_via_template_name() {
        // The renderer supplies the template name (adapters derive it from
        // the file stem); the core then strips ONE trailing ".at".
        let dir = tempfile_dir();
        let mut renderer = FakeRenderer::new();
        renderer
            .outputs
            .insert("double.at".to_string(), "x".to_string());
        let s = TemplateSpec {
            template_path: PathBuf::from("double.at.at"),
            output_name: None,
            rename: false,
        };
        let report = generate(
            &[s],
            &OutputConfig {
                output_dir: dir.clone(),
                ..Default::default()
            },
            &mut renderer,
        );
        assert!(report.errors.is_empty());
        assert_eq!(report.files_generated, vec![dir.join("double")]);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
