//! PLAN-938: one formatter, two passes. An allocation-free counting pass
//! admits the exact byte size with the host BEFORE the single exactly-sized
//! allocation, so produced text enters the request's cumulative output
//! account and the shared memory account before any payload byte exists.
//! The second pass renders into a bounded sink: a stateful renderer that
//! grew after its counting pass is refused, not silently reallocated.
use std::fmt;

struct Count(usize);
impl fmt::Write for Count {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        match self.0.checked_add(s.len()) {
            Some(total) => {
                self.0 = total;
                Ok(())
            }
            None => Err(fmt::Error),
        }
    }
}

struct Exact<'a> {
    out: &'a mut String,
    size: usize,
}
impl fmt::Write for Exact<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if self
            .out
            .len()
            .checked_add(s.len())
            .is_none_or(|n| n > self.size)
        {
            return Err(fmt::Error);
        }
        self.out.push_str(s);
        Ok(())
    }
}

/// Render through `render` twice: once counting (no allocation), then once
/// into one exactly pre-sized `String` admitted by `admit` in between.
pub(crate) fn render_admitted(
    render: impl Fn(&mut dyn fmt::Write) -> fmt::Result,
    admit: impl FnOnce(usize) -> Result<(), String>,
) -> Result<String, String> {
    let mut count = Count(0);
    render(&mut count).map_err(|_| "text size overflow".to_string())?;
    let size = count.0;
    admit(size)?;
    let mut out = String::new();
    out.try_reserve_exact(size)
        .map_err(|e| format!("text allocation: {e}"))?;
    render(&mut Exact {
        out: &mut out,
        size,
    })
    .map_err(|_| "text renderer changed after preflight".to_string())?;
    if out.len() != size {
        return Err("text renderer changed after preflight".into());
    }
    Ok(out)
}

/// The exact byte length of one render pass, without allocating.
pub(crate) fn text_len(render: impl FnOnce(&mut dyn fmt::Write) -> fmt::Result) -> usize {
    let mut count = Count(0);
    render(&mut count).expect("counting pass cannot fail");
    count.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lower::{print_fragment, print_fragment_admitted, CFn};
    use crate::runtime::{runtime_source, runtime_source_admitted};
    use ace_a2x_core::Ty;
    use std::cell::Cell;

    #[test]
    fn runtime_admission_preserves_bytes_and_the_certification_pin() {
        let sets: [&[&'static str]; 4] = [
            &[],
            &["hir.test.mark_a"],
            &["hir.test.mark_b"],
            &["hir.test.mark_a", "hir.test.mark_b"],
        ];
        for marks in sets {
            let legacy = runtime_source(marks);
            let mut seen = None;
            let admitted = runtime_source_admitted(marks, |size| {
                seen = Some(size);
                Ok(())
            })
            .unwrap();
            assert_eq!(admitted, legacy, "marks={marks:?}");
            assert_eq!(seen, Some(legacy.len()));
            assert_eq!(admitted.capacity(), legacy.len(), "not exactly pre-sized");
            let refused = runtime_source_admitted(marks, |_| Err("refused".into()));
            assert_eq!(refused.unwrap_err(), "refused");
        }
        // The explicit no-trace certification fixture pins this exact runtime
        // material (abi CERTIFICATION_C_RUNTIME_SHA256); renderer drift fails
        // here first, before any host qualification compares the pin.
        // PLAN-961: re-pinned — the C runtime gained the numeric helper
        // family (typed channels, checked matrix, shifts, conversions).
        assert_eq!(
            ace_a2x_core::sha256::sha256_hex(runtime_source(&[]).as_bytes()),
            "bd260a8b7d62f2b2956488fdeabffcf027605da89608ff81865cf8a8151de12d"
        );
    }

    #[test]
    fn fragment_admission_preserves_bytes_for_void_and_mixed_params() {
        let void_fn = CFn {
            name: "fn_main".into(),
            params: vec![],
            result_ty: Ty::I32,
            origin: (0, 42),
            body: vec![
                "a2x_result t0 = a2x_add((int32_t)20, (int32_t)22);".into(),
                "if (t0.trap != A2X_TRAP_NONE) { return t0; }".into(),
                "return a2x_value(t0.value);".into(),
            ],
        };
        let mixed = CFn {
            name: "fn_mix".into(),
            params: vec![
                ("int32_t".into(), "l0_a".into()),
                ("bool".into(), "l1_b".into()),
            ],
            result_ty: Ty::Bool,
            origin: (7, 99),
            body: vec!["return a2x_value((int32_t)(-2147483647 - 1));".into()],
        };
        for f in [&void_fn, &mixed] {
            let legacy = print_fragment(f);
            let mut seen = None;
            let admitted = print_fragment_admitted(f, |size| {
                seen = Some(size);
                Ok(())
            })
            .unwrap();
            assert_eq!(admitted, legacy);
            assert_eq!(seen, Some(legacy.len()));
            assert_eq!(admitted.capacity(), legacy.len(), "not exactly pre-sized");
            // The assembler's forward-declaration line keeps its shape.
            let params = if f.params.is_empty() {
                "void".to_string()
            } else {
                f.params
                    .iter()
                    .map(|(t, n)| format!("{t} {n}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            assert_eq!(
                legacy.lines().nth(1).unwrap(),
                format!("a2x_result {}({});", f.name, params)
            );
            let refused = print_fragment_admitted(f, |_| Err("refused".into())).unwrap_err();
            assert_eq!(refused.code, "a2c.foreign-resource");
        }
    }

    #[test]
    fn a_renderer_that_changes_after_its_counting_pass_is_refused() {
        // The guard targets STATEFUL renderers whose second pass would
        // exceed (or fall short of) the size the first pass declared.
        let grown = {
            let calls = Cell::new(0);
            render_admitted(
                |out| {
                    calls.set(calls.get() + 1);
                    if calls.get() == 1 {
                        out.write_str("ab")
                    } else {
                        out.write_str("abc")
                    }
                },
                |size| {
                    assert_eq!(size, 2);
                    Ok(())
                },
            )
        };
        assert_eq!(grown.unwrap_err(), "text renderer changed after preflight");
        let shrunk = {
            let calls = Cell::new(0);
            render_admitted(
                |out| {
                    calls.set(calls.get() + 1);
                    if calls.get() == 1 {
                        out.write_str("ab")
                    } else {
                        out.write_str("a")
                    }
                },
                |_| Ok(()),
            )
        };
        assert_eq!(shrunk.unwrap_err(), "text renderer changed after preflight");
    }
}
