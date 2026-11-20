//! Host-owned configuration for an explicitly prepared 967 provider.
//! Paths, expected hashes and approvals come from the SDK launcher, never
//! from the guest's request frame. The isolated child rebuilds the live
//! callable table from those facts.

use crate::protocol::hash_bytes;
use crate::{Failure, TARGET};
use ace_av_core::foreign_exec::ForeignStubTable;
#[cfg(windows)]
use ace_av_core::foreign_exec::{ForeignCallable, ForeignStub, ViewUtf8I32Fn};
use ace_av_core::mir::{MirInstr, MirModule};
use ace_foreign_contract::manual::{AdmittedForeign, HostAuthority, ManualManifest, SourceDecl};
use ace_foreign_contract::resource::{ResKind, ResOut, ResourceForm};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::io::Read;
use std::path::{Path, PathBuf};

const CONFIG_CAP: u64 = 256 * 1024;
#[cfg(windows)]
const PROVIDER_CAP: u64 = 64 * 1024 * 1024;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ForeignConfig {
    pub library: PathBuf,
    pub library_hash: [u8; 32],
    pub contract_digest: String,
    pub provider_digest: String,
    pub exports: BTreeSet<String>,
    pub manifest: ManualManifest,
}

pub(crate) struct Provider {
    #[cfg(windows)]
    _library: libloading::Library,
    pub(crate) stubs: ForeignStubTable,
    pub(crate) identity: String,
}

/// BLAKE3 of the exact host-owned configuration file. The SDK client
/// supplies the digest independently of the guest request.
pub struct ForeignPermit {
    pub(crate) config: ForeignConfig,
    pub(crate) hash: [u8; 32],
}

impl ForeignPermit {
    pub fn read(path: &Path, expected_hash: [u8; 32]) -> Result<Self, Failure> {
        if !path.is_absolute() {
            return Err(Failure::admission(
                "bridge.foreign-config-path",
                "expected an absolute SDK configuration path",
            ));
        }
        let file = std::fs::File::open(path)
            .map_err(|error| Failure::tool("bridge.foreign-config-read", error.to_string()))?;
        let mut bytes = Vec::new();
        file.take(CONFIG_CAP + 1)
            .read_to_end(&mut bytes)
            .map_err(|error| Failure::tool("bridge.foreign-config-read", error.to_string()))?;
        if bytes.len() as u64 > CONFIG_CAP {
            return Err(Failure::resource(
                "bridge.foreign-config-size",
                "SDK provider manifest exceeds its cap",
            ));
        }
        let hash = hash_bytes(&bytes);
        if hash != expected_hash {
            return Err(Failure::admission(
                "bridge.foreign-config-identity",
                "SDK provider configuration changed",
            ));
        }
        let config = serde_json::from_slice(&bytes).map_err(|error| {
            Failure::admission("bridge.foreign-config-shape", error.to_string())
        })?;
        Ok(Self { config, hash })
    }

    pub(crate) fn admit(
        &self,
        module: &MirModule,
        compile_time: bool,
    ) -> Result<AdmittedForeign, Failure> {
        let manifest = &self.config.manifest;
        if manifest.target != TARGET || !self.config.library.is_absolute() {
            return Err(Failure::admission(
                "bridge.foreign-host-domain",
                "provider must match the Windows x64 host ABI",
            ));
        }
        let authority = HostAuthority {
            review_id: manifest.review_id.clone(),
            provider: manifest.provider.clone(),
            header_digest: manifest.header_digest.clone(),
            physical_table_digest: manifest.physical_table_digest.clone(),
            target: TARGET.into(),
            provider_digest: self.config.provider_digest.clone(),
            contract_digest: self.config.contract_digest.clone(),
            capabilities: manifest.facts.capabilities.iter().cloned().collect(),
            available_symbols: manifest
                .declarations
                .iter()
                .flat_map(|declaration| {
                    std::iter::once(declaration.symbol.clone())
                        .chain(declaration.context_symbol.clone())
                })
                .collect(),
            binding_generation: 1,
        };
        let admitted = manifest
            .clone()
            .admit(&authority)
            .map_err(|error| Failure::admission("bridge.foreign-manifest", error))?;
        for decl in declarations(module) {
            if !self.config.exports.contains(&decl.function.symbol)
                || !admitted.admits_declaration(decl)
            {
                return Err(Failure::admission(
                    "bridge.foreign-declaration",
                    format!(
                        "{} is not in the approved provider export set",
                        decl.function.symbol
                    ),
                ));
            }
            if !compile_time {
                continue;
            }
            let pure = format!("pure:{}", decl.function.symbol);
            if !matches!(
                decl.resource(),
                Some(resource) if !resource.typed_peer && resource.params == [ResKind::ViewText]
                    && resource.form == (ResourceForm::Pure { ret: ResOut::I32 })
            ) || decl.context_symbol.is_some()
                || !decl.effects.iter().any(|effect| effect == &pure)
                || decl
                    .effects
                    .iter()
                    .any(|effect| effect != "checked" && effect != &pure)
            {
                return Err(Failure::admission(
                    "bridge.foreign-call-frame",
                    "CT external provider requires a reviewed pure view-text to I32 entry",
                ));
            }
        }
        Ok(admitted)
    }

    #[cfg(windows)]
    pub(crate) fn verified_image(&self) -> Result<(PathBuf, std::fs::File), Failure> {
        let path = self
            .config
            .library
            .canonicalize()
            .map_err(|error| Failure::tool("bridge.foreign-library", error.to_string()))?;
        use std::os::windows::fs::OpenOptionsExt;
        let image = std::fs::OpenOptions::new()
            .read(true)
            .share_mode(1)
            .open(&path)
            .map_err(|error| Failure::tool("bridge.foreign-library", error.to_string()))?;
        let found = provider_hash(&image)?;
        if found != self.config.library_hash {
            return Err(Failure::admission(
                "bridge.foreign-library-identity",
                "prepared provider image differs from the SDK receipt",
            ));
        }
        Ok((path, image))
    }

    #[cfg(windows)]
    pub(crate) fn load(&self, module: &MirModule) -> Result<Provider, Failure> {
        use libloading::os::windows::{
            Library, LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR, LOAD_LIBRARY_SEARCH_SYSTEM32,
        };
        let (path, _image) = self.verified_image()?;
        let found = self.config.library_hash;
        // Exact declaration and carrier checks are complete before LoadLibrary
        // can run the installed DLL's initialization.
        let library: libloading::Library = unsafe {
            Library::load_with_flags(
                &path,
                LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR | LOAD_LIBRARY_SEARCH_SYSTEM32,
            )
        }
        .map_err(|error| Failure::tool("bridge.foreign-library", error.to_string()))?
        .into();
        let mut table = ForeignStubTable::new(1);
        let mut bound = BTreeSet::new();
        for decl in declarations(module) {
            if !bound.insert(&decl.function.symbol) {
                continue;
            }
            let function = unsafe { library.get::<ViewUtf8I32Fn>(decl.function.symbol.as_bytes()) }
                .map_err(|error| Failure::tool("bridge.foreign-symbol", error.to_string()))?;
            table
                .insert(ForeignStub::new(
                    decl.provider.clone(),
                    decl.function.symbol.clone(),
                    decl.content_version.clone(),
                    1,
                    ForeignCallable::PureI32(*function),
                ))
                .map_err(|error| Failure::admission("bridge.foreign-stub", error))?;
        }
        Ok(Provider {
            _library: library,
            stubs: table,
            identity: format!(
                "foreign:{}:{}",
                self.config.manifest.provider,
                blake3::Hash::from(found)
            ),
        })
    }

    #[cfg(not(windows))]
    pub(crate) fn verified_image(&self) -> Result<(PathBuf, std::fs::File), Failure> {
        Err(Failure::admission(
            "bridge.foreign-host-domain",
            "native providers require Windows x64",
        ))
    }

    #[cfg(not(windows))]
    pub(crate) fn load(&self, _module: &MirModule) -> Result<Provider, Failure> {
        Err(Failure::admission(
            "bridge.foreign-host-domain",
            "the prepared external provider requires Windows x64",
        ))
    }
}

fn declarations(module: &MirModule) -> impl Iterator<Item = &SourceDecl> {
    module
        .funcs
        .iter()
        .flat_map(|function| &function.blocks)
        .flat_map(|block| &block.instrs)
        .filter_map(|instruction| match instruction {
            MirInstr::ForeignCall { declaration, .. } => Some(declaration.as_ref()),
            _ => None,
        })
}

#[cfg(windows)]
fn provider_hash(file: &std::fs::File) -> Result<[u8; 32], Failure> {
    let mut bounded = file.take(PROVIDER_CAP + 1);
    let mut hash = blake3::Hasher::new();
    std::io::copy(&mut bounded, &mut hash)
        .map_err(|error| Failure::tool("bridge.foreign-library", error.to_string()))?;
    if bounded.limit() == 0 {
        return Err(Failure::resource(
            "bridge.foreign-library-size",
            "provider image exceeds the configured size cap",
        ));
    }
    Ok(*hash.finalize().as_bytes())
}
