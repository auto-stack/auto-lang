use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::io::{self, Read, Write};
use std::time::Duration;

pub const REQUEST_CAP: usize = ace_av_core::contract::MAX_INPUT_BYTES;
pub const RESPONSE_CAP: usize = ace_native::config::MAX_REQUEST_OBJECTS_BYTES;
pub const BUILD_BUDGET: Duration = Duration::from_secs(5);
pub const CT_BUDGET: Duration = Duration::from_secs(10);
pub const TARGET: &str = ace_native::config::MN_TARGET;

const HEADER_BYTES: usize = 44;
// ABC and relocatable COFF are binary tails. Metadata has its own independent
// bound so a response-sized document cannot be handed to the JSON decoder.
const MAX_METADATA_BYTES: usize = 64 * 1024;
pub(crate) const REQUEST_MAGIC: [u8; 4] = *b"AQ01";
pub(crate) const RESPONSE_MAGIC: [u8; 4] = *b"AR01";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    NativeObjects,
    CompileTime,
}

impl Mode {
    pub fn budget(self) -> Duration {
        match self {
            Self::NativeObjects => BUILD_BUDGET,
            Self::CompileTime => CT_BUDGET,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Domains {
    pub build: String,
    pub host: String,
    pub target: String,
    pub abi: String,
    pub execution: String,
}

impl Domains {
    pub fn windows(mode: Mode) -> Self {
        let (abi, execution) = match mode {
            Mode::NativeObjects => (ace_native::config::MN_ABI, "native-object"),
            Mode::CompileTime => ("av-value/1", "hosted-av"),
        };
        Self {
            build: TARGET.into(),
            host: TARGET.into(),
            target: TARGET.into(),
            abi: abi.into(),
            execution: execution.into(),
        }
    }
}

pub type RequestId = [u8; 16];

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InputIdentity {
    pub source: [u8; 32],
    pub grammar: [u8; 32],
    pub passes: [u8; 32],
    pub stdlib: [u8; 32],
    pub dependencies: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireGlobal {
    pub name: String,
    pub ty: u32,
    pub mutable: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub id: RequestId,
    pub mode: Mode,
    pub domains: Domains,
    pub inputs: InputIdentity,
    pub owner: String,
    pub mir_schema: String,
    pub options: Options,
    pub capabilities: Vec<String>,
    pub globals: Vec<WireGlobal>,
    pub initializers: Vec<u32>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Options {
    /// The remaining budget of the caller's existing request (or CT root).
    pub remaining_ms: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureKind {
    Admission,
    Language,
    Resource,
    Cancelled,
    Tool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Failure {
    pub kind: FailureKind,
    pub code: String,
    pub message: String,
    pub span: Option<SourceSpan>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceSpan {
    pub function: u32,
    pub start: u32,
    pub end: u32,
}

impl From<ace_av_core::SpanInfo> for SourceSpan {
    fn from(span: ace_av_core::SpanInfo) -> Self {
        Self {
            function: span.func,
            start: span.start,
            end: span.end,
        }
    }
}

impl Failure {
    pub(crate) fn new(
        kind: FailureKind,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            code: code.into(),
            message: message.into(),
            span: None,
        }
    }

    pub(crate) fn admission(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(FailureKind::Admission, code, message)
    }

    pub(crate) fn resource(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(FailureKind::Resource, code, message)
    }

    pub(crate) fn tool(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(FailureKind::Tool, code, message)
    }
}

impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for Failure {}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Value {
    I32(i32),
    I64(i64),
    U64(u64),
    I8(i8),
    U8(u8),
    U32(u32),
    F32Bits(u32),
    F64Bits(u64),
    Bool(bool),
    Char(char),
    Nil,
    Unit,
    Text { len: u32 },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CtFacts {
    pub steps: u64,
    pub peak_slot_bytes: u64,
    pub root_frames: u64,
    pub live_text: u64,
    pub read_set: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Object {
    pub symbol: String,
    pub imports: Vec<(String, String)>,
    pub len: u32,
    pub hash: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum Outcome {
    Objects {
        entry: String,
        units: Vec<Object>,
        wrapper: Object,
        text_runtime: bool,
    },
    Value {
        value: Value,
        facts: CtFacts,
    },
    Failure {
        error: Failure,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Response {
    pub id: RequestId,
    pub request_hash: [u8; 32],
    pub outcome: Outcome,
}

pub(crate) struct Frame<T> {
    pub metadata: T,
    pub bytes: Vec<u8>,
    pub hash: [u8; 32],
}

fn lengths(metadata: usize, payload: usize, cap: usize) -> Result<(), Failure> {
    if metadata > MAX_METADATA_BYTES
        || metadata
            .checked_add(payload)
            .and_then(|len| len.checked_add(HEADER_BYTES))
            .is_none_or(|len| len > cap)
    {
        return Err(Failure::resource(
            "bridge.frame-size",
            format!("metadata {metadata} B and payload {payload} B exceed the frame budget"),
        ));
    }
    Ok(())
}

fn read_exact(input: &mut impl Read, bytes: &mut [u8]) -> Result<(), Failure> {
    input.read_exact(bytes).map_err(|error| {
        if error.kind() == io::ErrorKind::UnexpectedEof {
            Failure::admission(
                "bridge.frame-truncated",
                "frame ended before its declared length",
            )
        } else {
            Failure::tool("bridge.read", error.to_string())
        }
    })
}

fn body_hash(metadata: &[u8], payload: &[u8]) -> [u8; 32] {
    let mut hash = blake3::Hasher::new();
    hash.update(&(metadata.len() as u32).to_le_bytes());
    hash.update(&(payload.len() as u32).to_le_bytes());
    hash.update(metadata);
    hash.update(payload);
    *hash.finalize().as_bytes()
}

pub(crate) fn read_frame<T: DeserializeOwned>(
    mut input: impl Read,
    magic: [u8; 4],
    cap: usize,
) -> Result<Frame<T>, Failure> {
    let mut header = [0; HEADER_BYTES];
    read_exact(&mut input, &mut header)?;
    if header[..4] != magic {
        return Err(Failure::admission(
            "bridge.frame-version",
            "unknown service frame kind or version",
        ));
    }
    let metadata_len = u32::from_le_bytes(header[4..8].try_into().unwrap()) as usize;
    let payload_len = u32::from_le_bytes(header[8..12].try_into().unwrap()) as usize;
    lengths(metadata_len, payload_len, cap)?;
    let mut metadata = vec![0; metadata_len];
    let mut bytes = vec![0; payload_len];
    read_exact(&mut input, &mut metadata)?;
    read_exact(&mut input, &mut bytes)?;
    if input
        .read(&mut [0; 1])
        .map_err(|error| Failure::tool("bridge.read", error.to_string()))?
        != 0
    {
        return Err(Failure::admission(
            "bridge.frame-trailing",
            "one exchange permits exactly one frame",
        ));
    }
    let hash = body_hash(&metadata, &bytes);
    if header[12..] != hash {
        return Err(Failure::admission(
            "bridge.frame-checksum",
            "frame digest differs from content",
        ));
    }
    let metadata = serde_json::from_slice(&metadata)
        .map_err(|error| Failure::admission("bridge.frame-metadata", error.to_string()))?;
    Ok(Frame {
        metadata,
        bytes,
        hash,
    })
}

pub(crate) fn write_frame<T: Serialize>(
    mut output: impl Write,
    magic: [u8; 4],
    cap: usize,
    metadata: &T,
    payload: &[u8],
) -> Result<[u8; 32], Failure> {
    let metadata = serde_json::to_vec(metadata)
        .map_err(|error| Failure::tool("bridge.encode", error.to_string()))?;
    lengths(metadata.len(), payload.len(), cap)?;
    let hash = body_hash(&metadata, payload);
    for part in [
        magic.as_slice(),
        &(metadata.len() as u32).to_le_bytes(),
        &(payload.len() as u32).to_le_bytes(),
        &hash,
        &metadata,
        payload,
    ] {
        output
            .write_all(part)
            .map_err(|error| Failure::tool("bridge.write", error.to_string()))?;
    }
    output
        .flush()
        .map_err(|error| Failure::tool("bridge.write", error.to_string()))?;
    Ok(hash)
}

impl Outcome {
    pub(crate) fn verify_payload(&self, mut payload: &[u8]) -> Result<(), Failure> {
        let valid = match self {
            Self::Objects { units, wrapper, .. } => {
                for object in units.iter().chain([wrapper]) {
                    let len = object.len as usize;
                    let Some((bytes, remaining)) = payload.split_at_checked(len) else {
                        return Err(Failure::admission(
                            "bridge.response-object",
                            "truncated object payload",
                        ));
                    };
                    if len == 0 || hash_bytes(bytes) != object.hash {
                        return Err(Failure::admission(
                            "bridge.response-object",
                            "object digest differs from content",
                        ));
                    }
                    payload = remaining;
                }
                !units.is_empty() && payload.is_empty()
            }
            Self::Value {
                value: Value::Text { len },
                ..
            } => payload.len() == *len as usize && std::str::from_utf8(payload).is_ok(),
            Self::Value { .. } | Self::Failure { .. } => payload.is_empty(),
        };
        if valid {
            Ok(())
        } else {
            Err(Failure::admission(
                "bridge.response-value",
                "payload differs from the response declaration",
            ))
        }
    }
}

pub(crate) fn hash_bytes(bytes: &[u8]) -> [u8; 32] {
    *blake3::hash(bytes).as_bytes()
}
