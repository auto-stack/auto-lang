//! Language-owned compilation collector. It contains only owned declaration
//! data, never framework state, AST/Store objects, tasks, or VM references.
use super::messages::{HttpProgramMetadata, HttpRoute};
use std::{cell::RefCell, marker::PhantomData, rc::Rc, sync::Arc};

thread_local! {
    static COMPILING: RefCell<Option<HttpProgramMetadata>> = const { RefCell::new(None) };
}

/// Synchronous compilation scope, including dependency-module codegens. Freeze
/// it before executing/awaiting the program. Nesting and error unwinds restore
/// the previous collector; this guard cannot move to another thread.
pub struct CompilationMetadataGuard {
    previous: Option<HttpProgramMetadata>,
    active: bool,
    _local: PhantomData<Rc<()>>,
}

impl CompilationMetadataGuard {
    pub fn begin() -> Self {
        Self {
            previous: COMPILING.with(|slot| slot.replace(Some(HttpProgramMetadata::default()))),
            active: true,
            _local: PhantomData,
        }
    }

    pub fn finish(mut self, routes: &[(String, String, String)]) -> Arc<HttpProgramMetadata> {
        let mut metadata = COMPILING
            .with(|slot| slot.replace(self.previous.take()))
            .expect("compilation metadata collector missing");
        self.active = false;
        metadata.routes = routes
            .iter()
            .map(|(method, path, fn_name)| HttpRoute {
                method: method.clone(),
                path: path.clone(),
                fn_name: fn_name.clone(),
            })
            .collect();
        Arc::new(metadata)
    }
}

impl Drop for CompilationMetadataGuard {
    fn drop(&mut self) {
        if self.active {
            COMPILING.with(|slot| *slot.borrow_mut() = self.previous.take());
        }
    }
}

pub(crate) fn collect(update: impl FnOnce(&mut HttpProgramMetadata)) {
    COMPILING.with(|slot| {
        if let Some(metadata) = slot.borrow_mut().as_mut() {
            update(metadata);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::host_capabilities::http_service::VmHttpBridge;

    #[test]
    fn compilation_metadata_survives_nested_compile_error_and_unwind() {
        let outer = CompilationMetadataGuard::begin();
        VmHttpBridge::record_api_return_type("same", "outer");
        VmHttpBridge::record_api_return_shadowed("same");
        VmHttpBridge::record_api_param_sigs(
            "same",
            &[super::super::messages::ApiParamSig {
                name: "outer_parameter".into(),
                ty: "outer_type".into(),
            }],
        );
        VmHttpBridge::record_api_async_return("outer_async");
        let (vm, _, _, _) = crate::entry::create_vm_from_source(
            "#[api(method = \"GET\", path = \"/inner\")]\nfn same() int { 1 }",
        )
        .unwrap();
        let inner = VmHttpBridge::program_metadata(&vm).unwrap();
        assert_eq!(inner.returns["same"], "int");
        assert!(!inner.shadowed.contains("same"));
        assert!(inner.params["same"].is_empty());
        assert!(!inner.async_returns.contains("outer_async"));
        assert!(crate::entry::create_vm_from_source("fn broken(").is_err());
        let panic = std::panic::catch_unwind(|| {
            let _nested = CompilationMetadataGuard::begin();
            VmHttpBridge::record_api_return_type("same", "unwound");
            panic!("controlled compiler metadata unwind");
        });
        assert!(panic.is_err());
        let metadata = outer.finish(&[("GET".into(), "/outer".into(), "same".into())]);
        assert_eq!(metadata.returns["same"], "outer");
        assert!(metadata.shadowed.contains("same"));
        assert_eq!(metadata.params["same"][0].name, "outer_parameter");
        assert!(metadata.async_returns.contains("outer_async"));
        assert_eq!(metadata.routes[0].path, "/outer");
        COMPILING.with(|slot| assert!(slot.borrow().is_none()));
    }

    #[test]
    fn concurrent_real_compilers_freeze_only_their_own_declarations() {
        use std::sync::mpsc;
        use std::time::Duration;
        let (send, receive) = mpsc::channel();
        let mut releases = Vec::new();
        let mut workers = Vec::new();
        for index in 0..2 {
            let (release, released) = mpsc::channel();
            releases.push(release);
            let send = send.clone();
            workers.push(std::thread::spawn(move || {
                for iteration in 0..4 {
                    let ty = if index == 0 { "int" } else { "str" };
                    let later = if index == 0 { "fn later() ~int { delay_async(1) }" } else { "fn later() int { 240 }" };
                    let source = format!("#[api(method = \"GET\", path = \"/owner{index}/:n\")]\nfn same(n {ty}) {ty} {{ n }}\n{later}");
                    let (vm, _, _, _) = crate::entry::create_vm_from_source(&source).unwrap();
                    send.send((index, iteration, VmHttpBridge::program_metadata(&vm).unwrap())).unwrap();
                    // Both compilers must finish before either snapshot is
                    // checked. All cross-thread waits have explicit bounds.
                    released.recv_timeout(Duration::from_secs(10)).unwrap();
                }
            }));
        }
        drop(send);
        for iteration in 0..4 {
            let first = receive.recv_timeout(Duration::from_secs(10)).unwrap();
            let second = receive.recv_timeout(Duration::from_secs(10)).unwrap();
            assert_ne!(first.0, second.0);
            for (index, actual_iteration, metadata) in [first, second] {
                assert_eq!(actual_iteration, iteration);
                assert_eq!(
                    metadata.params["same"][0].ty,
                    if index == 0 { "int" } else { "str" }
                );
                assert_eq!(
                    metadata.returns["same"],
                    if index == 0 { "int" } else { "str" }
                );
                assert_eq!(metadata.async_returns.contains("later"), index == 0);
                assert_eq!(metadata.routes[0].path, format!("/owner{index}/:n"));
            }
            for release in &releases {
                release.send(()).unwrap();
            }
        }
        for worker in workers {
            worker.join().unwrap();
        }
        COMPILING.with(|slot| assert!(slot.borrow().is_none()));
    }
}
