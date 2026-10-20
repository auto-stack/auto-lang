//! R12-04 evidence probe: measures per-stage private-bytes of the frozen
//! 256 KiB load through the PUBLIC ac-core frontend seams (parse/resolve/
//! typecheck/compile-parts/descriptor-bind — each wrapped in a peak
//! sampler), then a full driver request. Run in RELEASE for evidence:
//! `mem_stage_probe [kb]` (size override uses the scaled load smoke).
//! Findings (2026-10-20, frozen load 273 575 B): parse+resolve+typecheck
//! ≤ ~20 MiB; generated atom_text 4 719 087 B; `descriptor::bind` ALONE
//! peaks at 166 649 856 B (≈158.9 MiB); full compile_parts_with
//! 209 444 864 B; full driver request ≈207 MiB — the atom render→re-bind
//! architecture inside the read-only ac-core public seam dominates and by
//! itself exceeds the 128 MiB worker budget (linear scaling ≈765 B per
//! source byte: 49/103/167/209 MiB at 60/136/197/274 KiB).
use auto_av_probe::mempeek::{sample_private_bytes, PeakSampler};

fn stage<F: FnOnce() -> T, T>(name: &str, f: F) -> T {
    let before = sample_private_bytes();
    let s = PeakSampler::start(2);
    let out = f();
    let peak = s.stop();
    let after = sample_private_bytes();
    println!(
        "{name}: before={before} peak={peak} after={after} delta_peak={}",
        peak.saturating_sub(before)
    );
    out
}

fn main() {
    use auto_ac_prototype::source::{
        parse_program_with, resolve_with, typecheck_with, SourceProfile,
    };
    let load = match std::env::args().nth(1) {
        Some(kb) => {
            let n: u32 = kb.parse().unwrap();
            auto_av_probe::load256::generate_with_sized(0, (n as usize) * 1024)
        }
        None => auto_av_probe::load256::generate(),
    };
    println!(
        "frozen load: {} bytes, expected {}",
        load.source.len(),
        load.expected_main
    );
    let idle = sample_private_bytes();
    println!("idle={idle}");

    // A: tokenize only
    let toks = stage("A-tokenize", || {
        auto_ac_prototype::source::lexer::tokenize(&load.source).unwrap()
    });
    println!("A: {} tokens", toks.len());
    drop(toks);

    // B..D: public stage seams, values kept alive like the real pipeline.
    let program = stage("B-parse", || {
        parse_program_with(&load.source, SourceProfile::Scalar).unwrap()
    });
    let resolved = stage("C-resolve", || {
        resolve_with(&program, &[], SourceProfile::Scalar).unwrap()
    });
    drop(program);
    stage("D-typecheck", || {
        typecheck_with(&resolved, SourceProfile::Scalar).unwrap()
    });
    drop(resolved);

    // E: full parts pipeline (includes the internal Atom emitter).
    let parts = stage("E-compile-parts", || {
        auto_ac_prototype::source::compile_parts_with(
            &load.source,
            "mem.at",
            &[],
            SourceProfile::Scalar,
        )
        .unwrap()
    });
    println!("E: atom_text={} bytes", parts.atom_text.len());
    drop(parts);

    // E2: the internal Atom re-bind alone (compile_parts re-parses and
    // binds the generated atom text through the descriptor binder).
    let src2 = load.source.clone();
    let atom = stage("E1-render-only", || {
        let program = parse_program_with(&src2, SourceProfile::Scalar).unwrap();
        let resolved = resolve_with(&program, &[], SourceProfile::Scalar).unwrap();
        let _ = typecheck_with(&resolved, SourceProfile::Scalar).unwrap();
        // render-only is not a public seam; approximate the emitter cost as
        // E-compile-parts minus bind below.
        let parts = auto_ac_prototype::source::compile_parts_with(
            &src2,
            "mem.at",
            &[],
            SourceProfile::Scalar,
        )
        .unwrap();
        parts.atom_text
    });
    let bundle = stage("E2-atom-bind", || {
        auto_ac_prototype::descriptor::bind(auto_ac_prototype::scalar_descriptor(), &atom).unwrap()
    });
    println!("E2: bundle modules={}", bundle.modules.len());
    drop(bundle);
    drop(atom);

    // F: full driver request (fresh, ephemeral — same shape as the R12 probe).
    let r = stage("F-driver-request", || {
        let mut d = auto_av_probe::driver::Driver::ephemeral();
        d.request(&load.source, "d_main", &[])
    });
    println!("F: value={:?} diag={:?}", r.value_i32(), r.diag);
    println!("final={}", sample_private_bytes());
}
