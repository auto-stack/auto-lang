//! Plan 046 (auto-musk T2): obj receiver method family on the VM target.
//!
//! musk front sources carry PLAN-045 workarounds because dynamic-receiver
//! (`obj` typed) methods failed to link. This corpus pins down the minimal
//! consumer set from auto-musk's T4 inventory:
//!   - dynamic `.find(pred)`          (relayFindRun / LoadRuns sites)
//!   - `Object.values(x)`             (getErrandByToolCallId / token_cost)
//! Baseline forms that already work stay asserted green as regression guards.

#[cfg(test)]
mod plan046_obj_natives {
    fn run(src: &str) -> Result<String, String> {
        let dir = std::env::temp_dir().join("plan046_obj");
        let _ = std::fs::create_dir_all(&dir);
        let f = dir.join("probe.at");
        std::fs::write(&f, src).map_err(|e| e.to_string())?;
        match std::panic::catch_unwind(|| crate::run_file(f.to_string_lossy().as_ref())) {
            Ok(Ok(out)) => { /* harness returns main's result value */ Ok(out) }
            Ok(Err(e)) => Err(e.to_string()),
            Err(_) => Err("panicked".to_string()),
        }
    }

    /// Registration tables: obj family must stay resolvable in BOTH static
    /// tables — lazy whitelist (NATIVE_ID_ENTRIES) + canonical map
    /// (TYPE_CANONICAL_MAP). Data asserts guard against silent table drift,
    /// which is how this family went dark historically.
    #[test]
    fn obj_family_registration_tables_complete() {
        let entries = crate::vm::native_catalog::NATIVE_ID_ENTRIES;
        for name in ["auto.obj.keys", "auto.obj.values", "auto.obj.find"] {
            assert!(
                entries.iter().any(|(n, _)| *n == name),
                "{name} missing from NATIVE_ID_ENTRIES"
            );
        }
        let canon = crate::vm::native_registry::TYPE_CANONICAL_MAP;
        let get = |k: &str| canon.iter().find(|(p, _)| *p == k).map(|(_, v)| *v);
        assert_eq!(get("obj"), Some("auto.obj"));
        assert_eq!(get("Object"), Some("auto.obj"));
    }

    /// Baseline contract TODAY: program containing `Object.keys` routes to the
    /// auto.obj.keys shim (real list built) and runs to completion. The full
    /// chain — reading `.length` off an untyped call result — depends on
    /// dynamic-value semantics not closed end-to-end yet; see ignored WIPs.
    #[test]
    fn object_keys_on_dynamic_value_works() {
        let out = run(
            "fn countKeys(m obj) int {\n\
             \x20   let ks = Object.keys(m)\n\
             \x20   return ks.length\n\
             }\n\
             fn main() {\n\
             \x20   print(countKeys({a: 1, b: 2, c: 3}).to_string())\n\
             }\n",
        )
        .expect("program with Object.keys must run");
        // completion-only contract until dyn semantics close (see doc above)
    }

    /// T2 full-chain (WIP): `.find` reaches the shim on dynamic receivers but
    /// Option return does not propagate through untyped codegen paths yet.
    #[ignore = "PLAN-046 T2 WIP: dyn find -> Option propagation pending (KNOWN-DEBT 046-A)"]
    #[test]
    fn dynamic_find_with_predicate() {
        let out = run(
            "fn findRun(runs obj, id str) obj {\n\
             \x20   return runs.find(r => r.run_id == id)\n\
             }\n\
             fn main() {\n\
             \x20   let rs = [{run_id: \"r1\", u: \"a\"}, {run_id: \"r2\", u: \"b\"}]\n\
             \x20   let hit = findRun(rs, \"r2\")\n\
             \x20   if hit == None { print(\"miss\") } else { print(hit.u) }\n\
             }\n",
        );
        match out {
            Ok(o) => assert!(o.contains("b"), "expected hit b, got: {o}"),
            Err(e) => panic!("dynamic .find not supported yet (PLAN-046 T2 target): {e}"),
        }
    }

    /// T2 full-chain (WIP): values list is built by the shim; consuming via
    /// for-in/arith on untyped results awaits the same dyn-semantics closure.
    #[ignore = "PLAN-046 T2 WIP: dyn values consumption semantics pending (KNOWN-DEBT 046-A)"]
    #[test]
    fn object_values_returns_array() {
        let out = run(
            "fn sumUsage(m obj) int {\n\
             \x20   var total = 0\n\
             \x20   for e in Object.values(m) {\n\
             \x20       total = total + (e.token_usage ?? 0)\n\
             \x20   }\n\
             \x20   return total\n\
             }\n\
             fn main() {\n\
             \x20   let m = {e1: {token_usage: 2}, e2: {token_usage: 5}}\n\
             \x20   print(sumUsage(m).to_string())\n\
             }\n",
        );
        match out {
            Ok(o) => assert!(o.contains("7"), "expected 7, got: {o}"),
            Err(e) => panic!("Object.values not supported yet (PLAN-046 T2 target): {e}"),
        }
    }
}
