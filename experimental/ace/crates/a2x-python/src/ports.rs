//! Canonical stdlib/IO operation port contract bindings for `ace-a2x-python`.
//!
//! Mirrors `stdlib/contracts/operations.toml` identity, parameter directions,
//! error contract, and explicit resource-handle lifecycle requirements so that
//! the Python runtime helper never invents ad-hoc names or hardcodes outcomes.

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
    let dep = format!("{}@{}", crate::runtime::PY_RUNTIME_VERSION, runtime_sha256);
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
        runtime_version: crate::runtime::PY_RUNTIME_VERSION,
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
