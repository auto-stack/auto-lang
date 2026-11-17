//! Canonical stdlib/IO operation port contract bindings for `ace-a2x-ts`.
//!
//! Mirrors `stdlib/contracts/operations.toml` identity, parameter directions,
//! error contract, and explicit resource-handle lifecycle requirements so that
//! the script runtime helper never invents ad-hoc names or hardcodes outcomes.
//!
//! The operation table below is target-neutral: the ids, semantic versions,
//! parameter directions and arity must stay identical to every other `a2x` leg
//! (the provider binds the same stdlib operations). Only the runtime identity
//! that binds the table (`crate::runtime::A2TS_RUNTIME_VERSION`) is
//! script-specific.

use ace_core::runtime_contract::sha256_hex;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ParamDirection {
    View,
    Mut,
    Move,
}

impl ParamDirection {
    pub fn as_str(self) -> &'static str {
        match self {
            ParamDirection::View => "View",
            ParamDirection::Mut => "Mut",
            ParamDirection::Move => "Move",
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PortParamSpec {
    pub ty: &'static str,
    pub direction: ParamDirection,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PortContract {
    pub operation_id: &'static str,
    pub semantic_version: u32,
    pub module: &'static str,
    pub symbol: &'static str,
    pub params: Vec<PortParamSpec>,
    pub returns: &'static str,
    pub error_contract: &'static str,
    pub dependency_identity: String,
}

impl PortContract {
    pub fn signature_tag(&self) -> String {
        let p: Vec<String> = self
            .params
            .iter()
            .map(|x| format!("{}:{}", x.direction.as_str(), x.ty))
            .collect();
        format!(
            "{}@v{}({})->{}",
            self.operation_id,
            self.semantic_version,
            p.join(","),
            self.returns
        )
    }
}

#[derive(Clone, Debug)]
pub struct StdlibPortCatalog {
    pub runtime_version: &'static str,
    pub runtime_sha256: String,
    pub ports: Vec<PortContract>,
}

impl StdlibPortCatalog {
    pub fn verify_call(
        &self,
        operation_id: &str,
        semantic_version: u32,
        directions: &[ParamDirection],
        runtime_sha256: &str,
    ) -> Result<&PortContract, String> {
        if runtime_sha256 != self.runtime_sha256 {
            return Err(format!(
                "port dependency identity mismatch for `{operation_id}`: expected {}, got {runtime_sha256}",
                self.runtime_sha256
            ));
        }
        let port = self
            .ports
            .iter()
            .find(|p| p.operation_id == operation_id)
            .ok_or_else(|| format!("unknown stdlib port `{operation_id}`"))?;
        if port.semantic_version != semantic_version {
            return Err(format!(
                "port `{operation_id}` semantic_version mismatch: expected {}, got {semantic_version}",
                port.semantic_version
            ));
        }
        if port.params.len() != directions.len()
            || port
                .params
                .iter()
                .zip(directions.iter())
                .any(|(spec, got)| spec.direction != *got)
        {
            return Err(format!(
                "port `{operation_id}` parameter direction/arity mismatch"
            ));
        }
        Ok(port)
    }

    /// Deterministic SHA-256 fingerprint of runtime + full ordered stdlib port contract table.
    pub fn canonical_fingerprint(&self) -> String {
        let mut buf = format!("{}|{}|", self.runtime_version, self.runtime_sha256);
        for p in &self.ports {
            buf.push_str(&p.signature_tag());
            buf.push(';');
        }
        sha256_hex(buf.as_bytes())
    }
}

/// Default canonical port catalog bound to the given runtime helper source SHA-256.
pub fn default_stdlib_port_catalog(runtime_source: &str) -> StdlibPortCatalog {
    let runtime_sha256 = sha256_hex(runtime_source.as_bytes());
    let dep = format!(
        "{}@{}",
        crate::runtime::A2TS_RUNTIME_VERSION,
        runtime_sha256
    );
    let mk = |operation_id, module, symbol, params, returns, error_contract| PortContract {
        operation_id,
        semantic_version: 1,
        module,
        symbol,
        params,
        returns,
        error_contract,
        dependency_identity: dep.clone(),
    };
    StdlibPortCatalog {
        runtime_version: crate::runtime::A2TS_RUNTIME_VERSION,
        runtime_sha256,
        ports: vec![
            mk(
                "math.private_square",
                "std.math",
                "square",
                vec![PortParamSpec {
                    ty: "int",
                    direction: ParamDirection::View,
                }],
                "int",
                "Trap(Overflow)",
            ),
            mk(
                "math.private_cube",
                "std.math",
                "cube",
                vec![PortParamSpec {
                    ty: "int",
                    direction: ParamDirection::View,
                }],
                "int",
                "Trap(Overflow)",
            ),
            mk(
                "text.byte_length",
                "std.text",
                "byte_len",
                vec![PortParamSpec {
                    ty: "str",
                    direction: ParamDirection::View,
                }],
                "int",
                "None",
            ),
            mk(
                "text.equal",
                "std.text",
                "equal",
                vec![
                    PortParamSpec {
                        ty: "str",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "str",
                        direction: ParamDirection::View,
                    },
                ],
                "bool",
                "None",
            ),
            mk(
                "text.clone",
                "std.text",
                "owned",
                vec![PortParamSpec {
                    ty: "str",
                    direction: ParamDirection::View,
                }],
                "Str",
                "Trap(Resource)",
            ),
            // PLAN-1041: the common output capability's platform-operation
            // ports (the a2x realization of the reserved stdlib platform
            // calls). Errors are deterministic negative i32 codes — the
            // same contract as the AV/AC backends — never traps.
            mk(
                "io.stdout",
                "std.platform",
                "io_stdout",
                vec![],
                "int",
                "None",
            ),
            mk(
                "io.stderr",
                "std.platform",
                "io_stderr",
                vec![],
                "int",
                "None",
            ),
            mk(
                "io.console_kind",
                "std.platform",
                "io_console_kind",
                vec![PortParamSpec {
                    ty: "int",
                    direction: ParamDirection::View,
                }],
                "int",
                "None",
            ),
            mk(
                "io.write_some_slice",
                "std.platform",
                "io_write_some_slice",
                vec![
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "str",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                ],
                "int",
                "None",
            ),
            mk(
                "io.console_write16",
                "std.platform",
                "io_console_write16",
                vec![
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                ],
                "int",
                "None",
            ),
            mk(
                "io.last_error",
                "std.platform",
                "io_last_error",
                vec![],
                "int",
                "None",
            ),
            // PLAN-973 system-resource ports (script target runtime)
            mk(
                "io.stdin",
                "std.platform",
                "io_stdin",
                vec![],
                "int",
                "None",
            ),
            mk(
                "mem.read_u8",
                "std.platform",
                "mem_read_u8",
                vec![
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                ],
                "int",
                "None",
            ),
            mk(
                "file.open_w16",
                "std.platform",
                "file_open_w16",
                vec![
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                ],
                "int",
                "None",
            ),
            mk(
                "file.open_bytes",
                "std.platform",
                "file_open_bytes",
                vec![
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                ],
                "int",
                "None",
            ),
            mk(
                "file.read_some",
                "std.platform",
                "file_read_some",
                vec![
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                ],
                "int",
                "None",
            ),
            mk(
                "file.write_some",
                "std.platform",
                "file_write_some",
                vec![
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                ],
                "int",
                "None",
            ),
            mk(
                "file.seek",
                "std.platform",
                "file_seek",
                vec![
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                ],
                "int",
                "None",
            ),
            mk(
                "file.tell",
                "std.platform",
                "file_tell",
                vec![PortParamSpec {
                    ty: "int",
                    direction: ParamDirection::View,
                }],
                "int",
                "None",
            ),
            mk(
                "file.hi",
                "std.platform",
                "file_hi",
                vec![PortParamSpec {
                    ty: "int",
                    direction: ParamDirection::View,
                }],
                "int",
                "None",
            ),
            mk(
                "file.size",
                "std.platform",
                "file_size",
                vec![PortParamSpec {
                    ty: "int",
                    direction: ParamDirection::View,
                }],
                "int",
                "None",
            ),
            mk(
                "file.mtime_ms",
                "std.platform",
                "file_mtime_ms",
                vec![PortParamSpec {
                    ty: "int",
                    direction: ParamDirection::View,
                }],
                "int",
                "None",
            ),
            mk(
                "file.flush",
                "std.platform",
                "file_flush",
                vec![PortParamSpec {
                    ty: "int",
                    direction: ParamDirection::View,
                }],
                "int",
                "None",
            ),
            mk(
                "file.close",
                "std.platform",
                "file_close",
                vec![PortParamSpec {
                    ty: "int",
                    direction: ParamDirection::View,
                }],
                "int",
                "None",
            ),
            mk(
                "file.path_attrs_w16",
                "std.platform",
                "file_path_attrs_w16",
                vec![
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                ],
                "int",
                "None",
            ),
            mk(
                "file.path_attrs_bytes",
                "std.platform",
                "file_path_attrs_bytes",
                vec![PortParamSpec {
                    ty: "int",
                    direction: ParamDirection::View,
                }],
                "int",
                "None",
            ),
            mk(
                "file.delete_w16",
                "std.platform",
                "file_delete_w16",
                vec![
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                ],
                "int",
                "None",
            ),
            mk(
                "file.delete_bytes",
                "std.platform",
                "file_delete_bytes",
                vec![PortParamSpec {
                    ty: "int",
                    direction: ParamDirection::View,
                }],
                "int",
                "None",
            ),
            mk(
                "file.dir_create_w16",
                "std.platform",
                "file_dir_create_w16",
                vec![
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                ],
                "int",
                "None",
            ),
            mk(
                "file.dir_create_bytes",
                "std.platform",
                "file_dir_create_bytes",
                vec![PortParamSpec {
                    ty: "int",
                    direction: ParamDirection::View,
                }],
                "int",
                "None",
            ),
            mk(
                "file.dir_delete_w16",
                "std.platform",
                "file_dir_delete_w16",
                vec![
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                ],
                "int",
                "None",
            ),
            mk(
                "file.dir_delete_bytes",
                "std.platform",
                "file_dir_delete_bytes",
                vec![PortParamSpec {
                    ty: "int",
                    direction: ParamDirection::View,
                }],
                "int",
                "None",
            ),
            mk(
                "file.replace_w16",
                "std.platform",
                "file_replace_w16",
                vec![
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                ],
                "int",
                "None",
            ),
            mk(
                "file.replace_bytes",
                "std.platform",
                "file_replace_bytes",
                vec![
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                ],
                "int",
                "None",
            ),
            mk(
                "file.canonical_w16",
                "std.platform",
                "file_canonical_w16",
                vec![
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                ],
                "int",
                "None",
            ),
            mk(
                "file.canonical_bytes",
                "std.platform",
                "file_canonical_bytes",
                vec![
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                ],
                "int",
                "None",
            ),
            mk(
                "file.dir_open_w16",
                "std.platform",
                "file_dir_open_w16",
                vec![
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                ],
                "int",
                "None",
            ),
            mk(
                "file.dir_next_w16",
                "std.platform",
                "file_dir_next_w16",
                vec![
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                ],
                "int",
                "None",
            ),
            mk(
                "file.dir_close",
                "std.platform",
                "file_dir_close",
                vec![PortParamSpec {
                    ty: "int",
                    direction: ParamDirection::View,
                }],
                "int",
                "None",
            ),
            mk(
                "file.dir_open_bytes",
                "std.platform",
                "file_dir_open_bytes",
                vec![PortParamSpec {
                    ty: "int",
                    direction: ParamDirection::View,
                }],
                "int",
                "None",
            ),
            mk(
                "file.dir_fill_bytes",
                "std.platform",
                "file_dir_fill_bytes",
                vec![
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                ],
                "int",
                "None",
            ),
            mk(
                "env.get_w16",
                "std.platform",
                "env_get_w16",
                vec![
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                ],
                "int",
                "None",
            ),
            mk(
                "env.set_w16",
                "std.platform",
                "env_set_w16",
                vec![
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                ],
                "int",
                "None",
            ),
            mk(
                "env.remove_w16",
                "std.platform",
                "env_remove_w16",
                vec![
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                ],
                "int",
                "None",
            ),
            mk(
                "env.get_bytes",
                "std.platform",
                "env_get_bytes",
                vec![
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                ],
                "int",
                "None",
            ),
            mk(
                "clock.unix_ms",
                "std.platform",
                "clock_unix_ms",
                vec![],
                "int",
                "None",
            ),
            mk(
                "clock.mono_ms",
                "std.platform",
                "clock_mono_ms",
                vec![],
                "int",
                "None",
            ),
            mk(
                "clock.hi",
                "std.platform",
                "clock_hi",
                vec![],
                "int",
                "None",
            ),
            mk(
                "clock.sleep_ms",
                "std.platform",
                "clock_sleep_ms",
                vec![PortParamSpec {
                    ty: "int",
                    direction: ParamDirection::View,
                }],
                "int",
                "None",
            ),
            mk(
                "entropy.rand_fill",
                "std.platform",
                "entropy_rand_fill",
                vec![
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                ],
                "int",
                "None",
            ),
            mk(
                "process.spawn_w16",
                "std.platform",
                "process_spawn_w16",
                vec![
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                ],
                "int",
                "None",
            ),
            mk(
                "process.spawn_bytes",
                "std.platform",
                "process_spawn_bytes",
                vec![
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                ],
                "int",
                "None",
            ),
            mk(
                "process.pipe_read",
                "std.platform",
                "process_pipe_read",
                vec![
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                ],
                "int",
                "None",
            ),
            mk(
                "process.wait",
                "std.platform",
                "process_wait",
                vec![
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                ],
                "int",
                "None",
            ),
            mk(
                "process.release",
                "std.platform",
                "process_release",
                vec![PortParamSpec {
                    ty: "int",
                    direction: ParamDirection::View,
                }],
                "int",
                "None",
            ),
            mk(
                "process.exit",
                "std.platform",
                "process_exit",
                vec![PortParamSpec {
                    ty: "int",
                    direction: ParamDirection::View,
                }],
                "int",
                "None",
            ),
            mk(
                "process.cmdline_w16",
                "std.platform",
                "process_cmdline_w16",
                vec![
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                ],
                "int",
                "None",
            ),
            mk(
                "process.cmdline_fill_bytes",
                "std.platform",
                "process_cmdline_fill_bytes",
                vec![
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                ],
                "int",
                "None",
            ),
            mk(
                "process.cwd_w16",
                "std.platform",
                "process_cwd_w16",
                vec![
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                ],
                "int",
                "None",
            ),
            mk(
                "process.cwd_bytes",
                "std.platform",
                "process_cwd_bytes",
                vec![
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                ],
                "int",
                "None",
            ),
            mk(
                "process.chdir_w16",
                "std.platform",
                "process_chdir_w16",
                vec![
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                ],
                "int",
                "None",
            ),
            mk(
                "process.chdir_bytes",
                "std.platform",
                "process_chdir_bytes",
                vec![PortParamSpec {
                    ty: "int",
                    direction: ParamDirection::View,
                }],
                "int",
                "None",
            ),
            mk(
                "mem.alloc",
                "std.platform",
                "mem_alloc",
                vec![PortParamSpec {
                    ty: "int",
                    direction: ParamDirection::View,
                }],
                "int",
                "None",
            ),
            mk(
                "mem.write_u8",
                "std.platform",
                "mem_write_u8",
                vec![
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                    PortParamSpec {
                        ty: "int",
                        direction: ParamDirection::View,
                    },
                ],
                "int",
                "None",
            ),
            mk(
                "mem.free",
                "std.platform",
                "mem_free",
                vec![PortParamSpec {
                    ty: "int",
                    direction: ParamDirection::View,
                }],
                "int",
                "None",
            ),
            mk(
                "console.say",
                "std.io",
                "say",
                vec![PortParamSpec {
                    ty: "str",
                    direction: ParamDirection::View,
                }],
                "unit",
                "Trap(PortIo)",
            ),
            mk(
                "io.read_line",
                "std.io",
                "read_line",
                vec![],
                "str",
                "Trap(PortIo)",
            ),
            mk(
                "io.read_line_opt",
                "std.io",
                "read_line_opt",
                vec![],
                "Option<str>",
                "Trap(PortIo)",
            ),
            mk(
                "io.open_buffer",
                "std.io",
                "open_buffer",
                vec![PortParamSpec {
                    ty: "str",
                    direction: ParamDirection::View,
                }],
                "Handle<BufferReader>",
                "Trap(Resource)",
            ),
            mk(
                "io.read_line_handle",
                "std.io",
                "read_line_handle",
                vec![PortParamSpec {
                    ty: "Handle<BufferReader>",
                    direction: ParamDirection::Mut,
                }],
                "Option<str>",
                "Trap(ResourceClosed|PortIo)",
            ),
            mk(
                "io.close_handle",
                "std.io",
                "close_handle",
                vec![PortParamSpec {
                    ty: "Handle<BufferReader>",
                    direction: ParamDirection::Move,
                }],
                "unit",
                "Trap(DoubleClose|Resource)",
            ),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Stand-in for the emitted script runtime text: the catalog only hashes
    /// it, so the exact body does not matter here.
    const RT_SRC: &str = "export const A2X_RUNTIME_VERSION = \"a2ts-runtime/1\";\n";
    const RT_SRC_ALT: &str = "export const A2X_RUNTIME_VERSION = \"a2ts-runtime/1\";\n// changed\n";

    fn catalog() -> StdlibPortCatalog {
        default_stdlib_port_catalog(RT_SRC)
    }

    #[test]
    fn canonical_fingerprint_is_stable_and_binds_runtime_sha() {
        let a = catalog();
        let b = catalog();
        // Deterministic: the same runtime source yields the same fragment key.
        assert_eq!(a.canonical_fingerprint(), b.canonical_fingerprint());
        assert_eq!(a.canonical_fingerprint().len(), 64);
        assert_eq!(a.runtime_version, crate::runtime::A2TS_RUNTIME_VERSION);

        // A different runtime sha must change the fingerprint.
        let c = default_stdlib_port_catalog(RT_SRC_ALT);
        assert_ne!(a.runtime_sha256, c.runtime_sha256);
        assert_ne!(a.canonical_fingerprint(), c.canonical_fingerprint());

        // Everything except the sha is unchanged, so the sha is what moved it.
        let mut d = a.clone();
        d.runtime_sha256 = c.runtime_sha256.clone();
        assert_eq!(d.ports.len(), a.ports.len());
        assert_eq!(d.canonical_fingerprint(), c.canonical_fingerprint());
        assert_ne!(d.canonical_fingerprint(), a.canonical_fingerprint());

        // Parity tripwire: the table is the target-neutral shared table.
        let mut ids: Vec<&str> = a.ports.iter().map(|p| p.operation_id).collect();
        assert_eq!(ids.len(), 71, "port table must keep a2x parity");
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), 71, "operation ids must be unique");
    }

    #[test]
    fn verify_call_accepts_console_say_and_rejects_wrong_direction() {
        let cat = catalog();
        let sha = cat.runtime_sha256.clone();

        let port = cat
            .verify_call("console.say", 1, &[ParamDirection::View], &sha)
            .expect("console.say@v1 declares one View parameter");
        assert_eq!(port.module, "std.io");
        assert_eq!(port.symbol, "say");
        assert_eq!(port.returns, "unit");
        assert_eq!(port.error_contract, "Trap(PortIo)");
        assert_eq!(port.signature_tag(), "console.say@v1(View:str)->unit");
        assert_eq!(
            port.dependency_identity,
            format!("{}@{}", cat.runtime_version, sha)
        );

        // Wrong direction on the same op.
        let err = cat
            .verify_call("console.say", 1, &[ParamDirection::Mut], &sha)
            .expect_err("console.say parameter is View, not Mut");
        assert!(err.contains("direction/arity"), "{err}");

        // Wrong arity (zero directions for a one-parameter op).
        let err = cat
            .verify_call("console.say", 1, &[], &sha)
            .expect_err("console.say takes exactly one parameter");
        assert!(err.contains("direction/arity"), "{err}");

        // Wrong direction on the second parameter of a two-parameter op.
        let err = cat
            .verify_call(
                "text.equal",
                1,
                &[ParamDirection::View, ParamDirection::Move],
                &sha,
            )
            .expect_err("text.equal declares View, View");
        assert!(err.contains("direction/arity"), "{err}");

        // A stale runtime sha is caught before the port lookup.
        let err = cat
            .verify_call("console.say", 1, &[ParamDirection::View], "00")
            .expect_err("runtime sha mismatch must reject");
        assert!(err.contains("dependency identity mismatch"), "{err}");
    }

    #[test]
    fn verify_call_rejects_unknown_op_id_and_wrong_semantic_version() {
        let cat = catalog();
        let sha = cat.runtime_sha256.clone();

        let err = cat
            .verify_call("io.no_such_port", 1, &[], &sha)
            .expect_err("unknown operation id must reject");
        assert!(
            err.contains("unknown stdlib port `io.no_such_port`"),
            "{err}"
        );

        let err = cat
            .verify_call("console.say", 2, &[ParamDirection::View], &sha)
            .expect_err("console.say only exists at v1");
        assert!(err.contains("semantic_version mismatch"), "{err}");

        let err = cat
            .verify_call("text.byte_length", 0, &[ParamDirection::View], &sha)
            .expect_err("no port exists at semantic version 0");
        assert!(err.contains("semantic_version mismatch"), "{err}");
    }
}
