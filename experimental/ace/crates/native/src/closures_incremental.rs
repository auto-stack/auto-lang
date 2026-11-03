//! PLAN-924 T-07: the native closures incremental session — the 912
//! retained single-function frontend driving REAL native production. A
//! lambda body edit lowers/verifies/emits EXACTLY the changed unit(s); an
//! owner-batch edit (creation-site facts changed) touches the owner plus
//! the truly-changed lambdas; unchanged units keep their installed COFF
//! objects and wire identities (whole-axis zero on supported routes). The
//! named non-fast routes rebuild the frontend whole, honestly counted.
//! Every revision is a REAL linked PE execution — no interpreter, no
//! cached run results.

use crate::closures_lower;
use crate::collections_runtime as runtime;
use crate::driver::collections::next_candidate;
use crate::link_driver;
use crate::observe::Obs;
use crate::MnError;
use ace_core::link::CaptureCaps;
use ace_av_core::mir::MirModule;
use ace_core::source::closures_fn::{
    ClosuresFnCandidate, ClosuresFunctionFrontend, NonFastReason,
};
use ace_core::{body_work, hir::DefKind};
use ace_core::verify::verify_closures_function;
use ace_lowering::lower_function_closures_unit_checked;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default)]
pub struct ClosuresWork {
    /// The REAL per-function frontend stages of this apply (body_work
    /// ledger): parse/resolve/typecheck/plan/bind/hir_verify/mir_lower/
    /// abc_encode/decode_admit per logical function, whole-axis entries as
    /// they run.
    pub bodies: std::collections::BTreeMap<String, ace_core::body_work::BodyWork>,
    pub obs: Obs,
    pub elapsed_ms: u128,
    pub route: String,
    pub submitted: Vec<String>,
    pub codegen_units: Vec<String>,
    pub relinked: bool,
}
pub struct ClosuresRun {
    pub exit_code: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub work: ClosuresWork,
}

pub struct ClosuresNativeSession {
    frontend: Option<ClosuresFunctionFrontend>,
    source_path: String,
    entry: String,
    wires: BTreeMap<String, Vec<u8>>,
    objects: BTreeMap<String, Vec<u8>>,
    symbols: BTreeMap<String, String>,
    imports: BTreeMap<String, Vec<(String, String)>>,
    runtime: runtime::Prepared,
    workdir: PathBuf,
    exe: Option<PathBuf>,
    knobs: crate::collections_lower::WrapperKnobs,
    manifest_id: Option<String>,
    closed: bool,
}
fn bad(message: impl Into<String>) -> MnError {
    MnError::Admit("closures-session", message.into())
}
fn frontend(errors: Vec<ace_core::atom_text::Diagnostic>) -> MnError {
    MnError::Unsupported("frontend", format!("{errors:?}"))
}

impl ClosuresNativeSession {
    /// Cold start: the retained 912 frontend (truth path) + first real PE
    /// execution. All subsequent edits ride the warm routes.
    pub fn new(
        source: &str,
        entry: &str,
        workdir: &Path,
        support: Option<&Path>,
    ) -> Result<(Self, ClosuresRun), MnError> {
        if source.len() > crate::config::MAX_INPUT_BYTES {
            return Err(MnError::Resource(
                "input-size",
                "closures source exceeds the native input budget".into(),
            ));
        }
        let started = std::time::Instant::now();
        let mut obs = Obs::default();
        let runtime = obs.span("runtime_prepare", |_| runtime::prepare(support))?;
        let (cold, cold_bodies) =
            body_work::capture_body_work(|| ClosuresFunctionFrontend::cold(source, "mn.at", &[]));
        let (frontend, _parts, checked) = cold.map_err(frontend)?;
        let units = obs.span("cold_lower", |_| Self::cold_units(&checked))?;
        // The entry must be an ordinary zero-argument int function.
        let entry_name = format!("d_{entry}");
        match units
            .iter()
            .find(|u| u.funcs[0].name == entry || u.funcs[0].name == entry_name)
        {
            None => return Err(bad("entry is missing")),
            Some(unit) => {
                let f = &unit.funcs[0];
                let sig = unit.sig_of(f.sig).expect("sig");
                if !sig.params.is_empty() || f.env.is_some() {
                    return Err(bad(
                        "the closures entry must be an ordinary zero-argument int function",
                    ));
                }
            }
        }
        let mut session = ClosuresNativeSession {
            frontend: Some(frontend),
            source_path: "mn.at".into(),
            entry: entry.into(),
            wires: BTreeMap::new(),
            objects: BTreeMap::new(),
            symbols: BTreeMap::new(),
            runtime,
            workdir: workdir.to_path_buf(),
            imports: BTreeMap::new(),
            exe: None,
            knobs: Default::default(),
            manifest_id: None,
            closed: false,
        };
        let mut names = Vec::new();
        for unit in units {
            let name = unit.funcs[0].name.clone();
            let wire = crate::closures_layout::unit_wire_bytes(&unit)?;
            session.wires.insert(name.clone(), wire);
            names.push((name, unit));
        }
        obs.counter("frontend_real_body_plans", names.len() as u64);
        let run = session.link_and_run(names, "cold", &mut obs, cold_bodies, started)?;
        Ok((session, run))
    }

    fn cold_units(
        checked: &ace_core::verify::CheckedModule,
    ) -> Result<Vec<MirModule>, MnError> {
        let bundle = checked.bundle();
        let module = bundle
            .modules
            .first()
            .ok_or_else(|| bad("closures bundle carries no module"))?;
        let snapshot = ace_av_core::contract::fnv1a64(b"mn.at");
        let mut units = Vec::new();
        for def in module
            .defs
            .iter()
            .filter(|def| matches!(def.kind, DefKind::Function { .. }))
        {
            body_work::record_attempt(ace_core::body_work::Stage::MirLower, &def.id_text);
            let unit = ace_lowering::hir_adapter::lower_function_closures_unit(
                bundle, &def.id_text, &[], snapshot,
            )
            .map_err(|e| MnError::Backend(format!("lower {}: {e:?}", def.id_text)))?;
            closures_lower::admit_layouts(&unit)?;
            units.push(unit);
        }
        Ok(units)
    }

    /// PLAN-924 V-08: assemble a warm session from VERIFIED restored
    /// material (the driver already re-admitted every MIR wire and matched
    /// every COFF hash/length/pages). The retained frontend is rebuilt from
    /// the restored source through the bounded 913 chain; the wire
    /// identities come from the restored bytes themselves; the first run
    /// links the restored objects with ZERO codegen (P0) over R>0 restored
    /// bytes. The next `apply_source` rides the ordinary warm routes — no
    /// live env, cell or code pointer is ever restored.
    pub fn from_restored(
        source: &str,
        entry: &str,
        workdir: &Path,
        support: Option<&Path>,
        restored: Vec<(MirModule, Vec<u8>)>,
        manifest_id: Option<String>,
    ) -> Result<(Self, ClosuresRun), MnError> {
        let started = std::time::Instant::now();
        let mut obs = Obs::default();
        let runtime = obs.span("runtime_prepare", |_| runtime::prepare(support))?;
        let cold =
            body_work::capture_body_work(|| ClosuresFunctionFrontend::cold(source, "mn.at", &[]));
        let (frontend, _parts, checked) = cold.0.map_err(frontend)?;
        let mut session = ClosuresNativeSession {
            frontend: Some(frontend),
            source_path: "mn.at".into(),
            entry: entry.into(),
            wires: BTreeMap::new(),
            objects: BTreeMap::new(),
            symbols: BTreeMap::new(),
            imports: BTreeMap::new(),
            runtime,
            workdir: workdir.to_path_buf(),
            exe: None,
            knobs: Default::default(),
            manifest_id,
            closed: false,
        };
        let bundle = checked.bundle();
        let module = bundle
            .modules
            .first()
            .ok_or_else(|| bad("closures bundle carries no module"))?;
        let declared: Vec<&str> = module
            .defs
            .iter()
            .filter(|def| matches!(def.kind, DefKind::Function { .. }))
            .map(|def| def.id_text.as_str())
            .collect();
        let entry_name = format!("d_{entry}");
        if !declared.iter().any(|n| *n == entry || *n == entry_name) {
            return Err(bad("entry is missing"));
        }
        let mut names = Vec::with_capacity(restored.len());
        for (unit, coff) in restored {
            let name = unit.funcs[0].name.clone();
            if !declared.contains(&name.as_str()) {
                return Err(bad(format!(
                    "restored unit {name} is not in the rebuilt source"
                )));
            }
            let wire = crate::closures_layout::unit_wire_bytes(&unit)?;
            session.wires.insert(name.clone(), wire);
            session
                .symbols
                .insert(name.clone(), crate::symbols::function_symbol(&unit.owner, &name));
            let mut imports = Vec::new();
            for import in &unit.imports {
                imports.push((
                    import.symbol.clone(),
                    crate::symbols::import_symbol(&unit.owner, &import.symbol)?,
                ));
            }
            session.imports.insert(name.clone(), imports);
            obs.counter("restored_coff_bytes_read", coff.len() as u64);
            obs.counter("cache_hit_coff", 1);
            session.objects.insert(name.clone(), coff);
            names.push((name, unit));
        }
        if names.len() != declared.len() {
            return Err(bad("restored units do not match the rebuilt function set"));
        }
        let run = session.link_and_run(Vec::new(), "restore", &mut obs, cold.1, started)?;
        Ok((session, run))
    }

    /// Apply a new full source: identical source is a ZERO-production hit
    /// (rerun); a single-region diff rides `edit_body`; other shapes (or a
    /// named non-fast reason) take the whole-module truth path.
    pub fn apply_source(&mut self, source: &str) -> Result<ClosuresRun, MnError> {
        self.apply_knobs(source, self.knobs)
    }

    pub fn apply_knobs(
        &mut self,
        source: &str,
        knobs: crate::collections_lower::WrapperKnobs,
    ) -> Result<ClosuresRun, MnError> {
        assert!(!self.closed, "session closed");
        let started = std::time::Instant::now();
        let mut obs = Obs::default();
        let front = self
            .frontend
            .as_ref()
            .ok_or_else(|| bad("no retained frontend"))?;
        let old = front.source().to_string();
        if old == source {
            // Zero-production hit: no frontend stage, no codegen, no relink
            // when the knobs are unchanged — the installed PE re-runs.
            if knobs == self.knobs {
                if let Some(exe) = self.exe.clone() {
                    let run = self.rerun(&exe, &mut obs, "hit")?;
                    let mut work = ClosuresWork {
                        route: "hit".into(),
                        ..Default::default()
                    };
                    work.obs = obs;
                    work.elapsed_ms = started.elapsed().as_millis();
                    return Ok(ClosuresRun {
                        exit_code: run.0,
                        stdout: run.1,
                        stderr: run.2,
                        work,
                    });
                }
            }
            let run = self.link_and_run(Vec::new(), "hit-relink", &mut obs, Default::default(), started)?;
            self.knobs = knobs;
            return Ok(run);
        }
        body_work::record_whole(ace_core::body_work::Stage::FragmentDiff);
        let (range, replacement) = diff_single_region(&old, source);
        let base_sha = front.source_sha().to_string();
        match self
            .frontend
            .as_mut()
            .expect("frontend present")
            .edit_body(&base_sha, range, &replacement)
        {
            Ok(Ok(candidate)) => self.apply_fast(candidate, knobs, &mut obs, started),
            Ok(Err(reason)) => self.apply_nonfast(source, reason, knobs, &mut obs, started),
            Err(ds) => Err(frontend(ds)),
        }
    }

    /// The warm route: verify every re-resolved unit against the (overlaid)
    /// retained registry → lower from VERIFIED facts → spanless wire →
    /// submit exactly the changed units; the frontend commits only after the
    /// real PE run succeeded.
    fn apply_fast(
        &mut self,
        candidate: ClosuresFnCandidate,
        knobs: crate::collections_lower::WrapperKnobs,
        obs: &mut Obs,
        started: std::time::Instant,
    ) -> Result<ClosuresRun, MnError> {
        let front = self
            .frontend
            .as_ref()
            .expect("warm implies a retained frontend");
        let registry_after = if candidate.owner.is_some() {
            Some(
                front
                    .registry()
                    .overlay_from_batch(&candidate.parts.bundle, &candidate.real_def_ids())
                    .map_err(frontend)?,
            )
        } else {
            None
        };
        let ctx = registry_after.as_ref().unwrap_or_else(|| front.registry());
        let snapshot = ace_av_core::contract::fnv1a64(self.source_path.as_bytes());
        let mut lowered: Vec<(String, MirModule, Vec<u8>)> = Vec::new();
        let mut facts: Vec<ace_core::source::closures_fn::UnitCommitFacts> = Vec::new();
        for unit in &candidate.units {
            let wire_name = candidate.def_id_text_of(unit);
            let credential = verify_closures_function(
                &candidate.parts.bundle,
                candidate.def_index_of(unit),
                ctx,
            )
            .map_err(frontend)?;
            if credential.def_id_text() != wire_name {
                return Err(bad(format!(
                    "credential `{}` does not name the edited unit `{wire_name}`",
                    credential.def_id_text()
                )));
            }
            facts.push(ace_core::source::closures_fn::UnitCommitFacts {
                fn_index: unit.fn_index,
                body_digest: credential.body_digest().to_string(),
                context_revision: credential.context_revision().to_string(),
            });
            body_work::record_attempt(ace_core::body_work::Stage::MirLower, &wire_name);
            let mir = lower_function_closures_unit_checked(
                &candidate.parts.bundle,
                &credential,
                &[],
                snapshot,
            )
            .map_err(|e| MnError::Backend(format!("lower {wire_name}: {e:?}")))?;
            body_work::record_attempt(ace_core::body_work::Stage::AbcEncode, &wire_name);
            let wire = crate::closures_layout::unit_wire_bytes(&mir)?;
            closures_lower::admit_layouts(&mir)?;
            lowered.push((wire_name, mir, wire));
        }
        let changed: Vec<(String, MirModule)> = lowered
            .iter()
            .filter(|(name, _, wire)| self.wires.get(name) != Some(wire))
            .map(|(name, mir, _)| (name.clone(), mir.clone()))
            .collect();
        let route = self.route_text(&candidate);
        let run = self.link_and_run(changed, &route, obs, Default::default(), started)?;
        // The real run succeeded — install the frontend revision and wires.
        self.frontend
            .as_mut()
            .expect("frontend present")
            .commit_candidate(candidate, registry_after, &facts)
            .map_err(|e| bad(format!("commit candidate: {e:?}")))?;
        for (name, _, wire) in lowered {
            self.wires.insert(name, wire);
        }
        let mut with_knobs = run;
        self.knobs = knobs;
        with_knobs.work.route = route;
        Ok(with_knobs)
    }

    fn apply_nonfast(
        &mut self,
        source: &str,
        reason: NonFastReason,
        knobs: crate::collections_lower::WrapperKnobs,
        obs: &mut Obs,
        started: std::time::Instant,
    ) -> Result<ClosuresRun, MnError> {
        let route = format!("nonfast:{}", non_fast_text(&reason));
        let (frontend, _parts, checked) = match self.frontend.as_ref() {
            Some(prev) => ClosuresFunctionFrontend::cold_carrying(
                source,
                &self.source_path,
                &[],
                prev,
            ),
            None => ClosuresFunctionFrontend::cold(source, &self.source_path, &[]),
        }
        .map_err(frontend)?;
        let units = Self::cold_units(&checked)?;
        let mut changed: Vec<(String, MirModule)> = Vec::new();
        let mut next_wires = self.wires.clone();
        for unit in units {
            let name = unit.funcs[0].name.clone();
            let wire = crate::closures_layout::unit_wire_bytes(&unit)?;
            if self.wires.get(&name) != Some(&wire) {
                changed.push((name.clone(), unit));
            }
            next_wires.insert(name, wire);
        }
        self.wires = next_wires;
        let run = self.link_and_run(changed, &route, obs, Default::default(), started)?;
        self.frontend = Some(frontend);
        self.knobs = knobs;
        let mut with_route = run;
        with_route.work.route = route;
        Ok(with_route)
    }

    fn route_text(&self, candidate: &ClosuresFnCandidate) -> String {
        match &candidate.owner {
            None => format!("fast-unit:{}", candidate.def_id_text()),
            Some(owner) => {
                let changed: Vec<String> = candidate
                    .units
                    .iter()
                    .skip(1)
                    .map(|u| candidate.def_id_text_of(u))
                    .collect();
                let reused: Vec<String> = owner
                    .sites
                    .iter()
                    .filter(|(_, site)| owner.reused_ordinals.contains(&site.ordinal))
                    .map(|(_, site)| format!("d_{}", site.name))
                    .collect();
                format!(
                    "fast-owner:{} changed={:?} reused={:?}",
                    candidate.def_id_text(),
                    changed,
                    reused
                )
            }
        }
    }

    /// Emit EXACTLY the changed units, relink the full PE and run it. An
    /// empty change set with no installed exe links everything (cold).
    fn link_and_run(
        &mut self,
        changed: Vec<(String, MirModule)>,
        route: &str,
        obs: &mut Obs,
        bodies: std::collections::BTreeMap<String, ace_core::body_work::BodyWork>,
        started: std::time::Instant,
    ) -> Result<ClosuresRun, MnError> {
        let mut codegen_units = Vec::new();
        for (name, unit) in changed {
            let object = obs.span("codegen", |o| {
                o.counter("codegen_emit_units", 1);
                closures_lower::emit_unit_coff(&unit)
            })?;
            self.objects.insert(name.clone(), object.bytes.clone());
            self.symbols.insert(name.clone(), object.symbol.clone());
            self.imports.insert(name.clone(), object.imports.clone());
            codegen_units.push(name);
        }
        let mut inputs = Vec::with_capacity(self.objects.len());
        let mut names: Vec<&String> = self.objects.keys().collect();
        names.sort();
        for name in names {
            let bytes = self.objects.get(name).expect("object present").clone();
            inputs.push(link_driver::ObjectInput {
                name: name.clone(),
                bytes,
            });
        }
        let entry_symbol = self.entry_symbol()?;
        let wrapper = obs.span("codegen", |o| {
            o.counter("codegen_emit_wrappers", 1);
            crate::collections_lower::emit_wrapper_coff(&entry_symbol, &self.knobs)
        })?;
        std::fs::create_dir_all(&self.workdir)
            .map_err(|e| MnError::Infra(e.to_string()))?;
        let candidate_dir = self.workdir.join(format!(
            ".closures-session-{}-{}",
            std::process::id(),
            next_candidate()
        ));
        std::fs::create_dir(&candidate_dir)
            .map_err(|e| MnError::Infra(e.to_string()))?;
        let linked = obs.span("link", |o| {
            o.counter("link_invocations", 1);
            link_driver::link_only(
                &inputs,
                &wrapper,
                crate::config::START_SYMBOL,
                &candidate_dir,
                &[self.runtime.library.clone()],
                &format!("closures session {route}\n"),
            )
        });
        let linked = match linked {
            Ok(l) => l,
            Err(e) => {
                let _ = std::fs::remove_dir_all(candidate_dir);
                return Err(e);
            }
        };
        let (exit, stdout, stderr) = match self.run_and_decode(&linked.exe_path, obs) {
            Ok(r) => r,
            Err(e) => {
                let _ = std::fs::remove_dir_all(candidate_dir);
                return Err(e);
            }
        };
        let published = link_driver::publish_collections(&linked, &self.workdir);
        let _ = std::fs::remove_dir_all(candidate_dir);
        let published = published?;
        // The hit route re-runs the PUBLISHED exe (workdir/collections.exe),
        // not the candidate-dir product that publish just moved away.
        self.exe = Some(published.exe_path.clone());
        crate::collections_memory::check()?;
        let mut work = ClosuresWork {
            bodies,
            obs: obs.clone(),
            elapsed_ms: started.elapsed().as_millis(),
            route: route.to_string(),
            submitted: codegen_units.clone(),
            codegen_units,
            relinked: true,
        };
        for (i, name) in crate::resources::PARTITION_NAMES.iter().enumerate() {
            work.obs
                .counter(&format!("ledger_peak_{name}"), obs.ledger.peaks()[i]);
        }
        Ok(ClosuresRun {
            exit_code: exit,
            stdout,
            stderr,
            work,
        })
    }

    fn rerun(
        &mut self,
        exe: &Path,
        obs: &mut Obs,
        route: &str,
    ) -> Result<(Option<i32>, Vec<u8>, Vec<u8>), MnError> {
        let (exit, stdout, stderr) = self.run_and_decode(exe, obs)?;
        crate::collections_memory::check()?;
        let _ = route;
        Ok((exit, stdout, stderr))
    }

    fn run_and_decode(
        &mut self,
        exe: &Path,
        obs: &mut Obs,
    ) -> Result<(Option<i32>, Vec<u8>, Vec<u8>), MnError> {
        let run = obs.span("run", |o| {
            o.counter("run_invocations", 1);
            link_driver::run_pe(
                exe,
                crate::config::RUN_BUDGET,
                CaptureCaps {
                    stdout_max: crate::config::STREAM_CAP,
                    stderr_max: crate::config::STREAM_CAP,
                },
            )
        })?;
        let observed = crate::collections_observe::decode(run.exit_code, &run.stderr)?;
        use crate::collections_observe::Outcome;
        let error = match observed.outcome {
            Outcome::Trap(22) => Some(MnError::Resource(
                "execution-budget",
                "native closure allocation/budget refused".into(),
            )),
            Outcome::Trap(23) => Some(MnError::Admit(
                "runtime-value",
                "invalid native closure value/layout".into(),
            )),
            Outcome::Trap(25) => Some(MnError::Resource(
                "allocation-injected",
                "native allocation fault seam fired".into(),
            )),
            Outcome::Trap(26) => Some(MnError::Resource(
                "cancelled",
                "native closure operation cancelled before commit".into(),
            )),
            _ => None,
        };
        if let Some(error) = error {
            return Err(error);
        }
        Ok((run.exit_code, run.stdout, run.stderr))
    }

    fn entry_symbol(&self) -> Result<String, MnError> {
        let by_name = format!("d_{}", self.entry);
        let mut hits = self
            .symbols
            .iter()
            .filter(|(name, _)| name.as_str() == self.entry || **name == by_name);
        let hit = hits.next().ok_or_else(|| bad("entry is missing"))?;
        if hits.next().is_some() {
            return Err(bad("entry is ambiguous"));
        }
        Ok(hit.1.clone())
    }

    pub fn close(&mut self) -> Result<(), MnError> {
        if let Some(frontend) = self.frontend.as_mut() {
            frontend.close();
        }
        self.closed = true;
        Ok(())
    }

    /// The persisted manifest this session was restored from (None in
    /// readonly mode or for a purely ephemeral session).
    pub fn manifest_id(&self) -> Option<&str> {
        self.manifest_id.as_deref()
    }

    /// SHA-256 of every resident unit object — the codegen identity a
    /// consumer diffes to prove unchanged units kept their linked COFF.
    pub fn object_hashes(&self) -> BTreeMap<String, String> {
        self.objects
            .iter()
            .map(|(name, bytes)| (name.clone(), crate::cache::content_hash(bytes)))
            .collect()
    }
}

/// The single-region literal diff (M axis: metered as source_hash +
/// fragment_diff in the frontend ledger).
fn diff_single_region(old: &str, new: &str) -> (std::ops::Range<usize>, String) {
    let a = old.as_bytes();
    let b = new.as_bytes();
    let mut start = 0;
    while start < a.len() && start < b.len() && a[start] == b[start] {
        start += 1;
    }
    let mut end_a = a.len();
    let mut end_b = b.len();
    while end_a > start && end_b > start && a[end_a - 1] == b[end_b - 1] {
        end_a -= 1;
        end_b -= 1;
    }
    let range = std::ops::Range {
        start: char_boundary(old, start),
        end: char_boundary(old, end_a),
    };
    let replacement = String::from_utf8_lossy(&b[start..end_b]).into_owned();
    (range, replacement)
}
fn char_boundary(s: &str, mut i: usize) -> usize {
    if i > s.len() {
        i = s.len();
    }
    while i > 0 && !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

fn non_fast_text(reason: &NonFastReason) -> String {
    match reason {
        NonFastReason::OwnerSignatureChanged => "owner-signature-changed".into(),
        NonFastReason::OwnerTopologyChanged { .. } => "owner-topology-changed".into(),
        NonFastReason::CrossesUnits => "crosses-units".into(),
        NonFastReason::StaleBase => "stale-base".into(),
        NonFastReason::LambdaSiteChanged(inner) => format!("lambda-site-changed:{inner:?}"),
        NonFastReason::LambdaNested => "lambda-nested".into(),
        NonFastReason::RestoredContext => "restored-context".into(),
    }
}
