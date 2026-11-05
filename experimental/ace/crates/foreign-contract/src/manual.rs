//! PLAN-938: untrusted manual source declarations and current host admission.
//!
//! A manifest is never authority. The compilation host supplies an independently
//! reviewed contract digest and CURRENT provider/header identities. No addresses
//! or persisted safety credentials occur in this model.
use crate::{FnForm, InterfaceFacts, LaneKind, PhysicalPlan};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

pub const MANUAL_SCHEMA: &str = "ace.foreign.manual/1";
pub const SOURCE_CAP: &str = "foreign.scalar";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManualDecl {
    pub source_name: String,
    pub symbol: String,
    pub context_symbol: Option<String>,
}

/// Untrusted per-declaration facts carried by HIR/MIR. Possession is not a
/// credential: a consumer must compare them with the current admitted host
/// environment. No provider implementation digest enters this signature key.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceDecl {
    pub name: String,
    pub provider: String,
    pub logical: String,
    pub content_version: String,
    pub review_id: String,
    pub header_digest: String,
    pub physical_table_digest: String,
    pub context_symbol: Option<String>,
    pub function: crate::FnFact,
    pub effects: Vec<String>,
    pub capabilities: Vec<String>,
}

impl SourceDecl {
    /// Heap allocations owned by this declaration, excluding the inline
    /// SourceDecl itself. Measurement is allocation-free, including malformed
    /// candidates. Containers charge their own inline/boxed storage separately.
    pub fn heap_bytes(&self) -> usize {
        let mut bytes = 0usize;
        let mut add = |n: usize| {
            bytes = bytes.saturating_add(n);
        };
        for text in [
            &self.name,
            &self.provider,
            &self.logical,
            &self.content_version,
            &self.review_id,
            &self.header_digest,
            &self.physical_table_digest,
            &self.function.symbol,
        ] {
            add(text.capacity());
        }
        if let Some(context) = &self.context_symbol {
            add(context.capacity());
        }
        match &self.function.form {
            FnForm::Pure { params, .. } => add(params
                .capacity()
                .saturating_mul(std::mem::size_of::<LaneKind>())),
            FnForm::Fallible { params, outs, .. } => {
                add(params
                    .capacity()
                    .saturating_mul(std::mem::size_of::<LaneKind>()));
                add(outs
                    .capacity()
                    .saturating_mul(std::mem::size_of::<LaneKind>()));
            }
        }
        if let Some(error) = &self.function.error_model {
            add(error.kind.capacity());
            add(error
                .codes
                .capacity()
                .saturating_mul(std::mem::size_of::<(i32, String)>()));
            for (_, reason) in &error.codes {
                add(reason.capacity());
            }
        }
        if let Some((id, version)) = &self.function.semantic_ref {
            add(id.capacity());
            add(version.capacity());
        }
        for values in [&self.effects, &self.capabilities] {
            add(values
                .capacity()
                .saturating_mul(std::mem::size_of::<String>()));
            for value in values {
                add(value.capacity());
            }
        }
        bytes
    }
    pub fn validate(&self) -> Result<(), String> {
        manual_facts(
            std::slice::from_ref(&self.function),
            &self.effects,
            &self.capabilities,
        )?;
        let needs_context = matches!(self.function.form, FnForm::Fallible { ctx: Some(_), .. });
        if needs_context != self.context_symbol.is_some()
            || self.context_symbol.as_ref().is_some_and(|s| !ident(s))
        {
            return Err("foreign.context: invalid target context binding".into());
        }
        if !ident(&self.name)
            || !ident(&self.function.symbol)
            || self.logical.is_empty()
            || self.content_version.is_empty()
            || self.provider.is_empty()
            || self.review_id.is_empty()
            || !digest(&self.header_digest)
            || !digest(&self.physical_table_digest)
        {
            return Err("foreign.declaration: invalid identity/provenance".into());
        }
        let scalar = |k: &LaneKind| matches!(k, LaneKind::I32 | LaneKind::Bool);
        let valid = match &self.function.form {
            FnForm::Pure { params, ret } => params.iter().all(scalar) && scalar(ret),
            FnForm::Fallible { params, outs, .. } => {
                params.iter().all(scalar) && outs.len() == 1 && outs.iter().all(scalar)
            }
        };
        if !valid {
            return Err("foreign.signature: expected finite scalar signature".into());
        }
        InterfaceFacts {
            format: crate::FACTS_FORMAT.into(),
            logical: self.logical.clone(),
            content_version: self.content_version.clone(),
            source: crate::FactsSource::Typed {
                id: self.review_id.clone(),
            },
            functions: vec![self.function.clone()],
            effects: self.effects.clone(),
            capabilities: self.capabilities.clone(),
        }
        .validate()
    }

    pub fn digest(&self) -> String {
        // Hash validation streams the wire identity; it does not materialize
        // or regenerate a persisted artifact during recovery.
        struct Hash(blake3::Hasher);
        impl std::io::Write for Hash {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                self.0.update(bytes);
                Ok(bytes.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let mut hash = Hash(blake3::Hasher::new());
        serde_json::to_writer(&mut hash, self).expect("serializable declaration");
        hash.0.finalize().to_hex().to_string()
    }

    pub fn params(&self) -> &[LaneKind] {
        match &self.function.form {
            FnForm::Pure { params, .. } | FnForm::Fallible { params, .. } => params,
        }
    }

    pub fn result(&self) -> LaneKind {
        match &self.function.form {
            FnForm::Pure { ret, .. } => *ret,
            FnForm::Fallible { outs, .. } => outs[0],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManualManifest {
    pub schema: String,
    pub review_id: String,
    pub provider: String,
    pub header_digest: String,
    pub physical_table_digest: String,
    pub target: String,
    pub profile: String,
    pub callconv: String,
    pub direction: String,
    pub facts: InterfaceFacts,
    pub declarations: Vec<ManualDecl>,
}

/// Process-local host input. This must come from the compilation host's current
/// review registry and loaded provider, not from a field beside the manifest.
#[derive(Clone, Debug)]
pub struct HostAuthority {
    pub review_id: String,
    pub contract_digest: String,
    pub provider: String,
    pub provider_digest: String,
    pub header_digest: String,
    pub physical_table_digest: String,
    pub target: String,
    pub capabilities: BTreeSet<String>,
    pub available_symbols: BTreeSet<String>,
    pub binding_generation: u64,
}

/// Intentionally not serializable. Construction requires current host admission.
#[derive(Clone, Debug)]
pub struct AdmittedForeign {
    manifest: Arc<ManualManifest>,
    plan: Arc<PhysicalPlan>,
    contract_digest: String,
    provider_digest: String,
    generation: u64,
    declarations: Arc<Vec<SourceDecl>>,
    declaration_digests: Arc<BTreeMap<String, String>>,
    pages: ForeignPages,
    allocation_identity: Arc<()>,
}

/// Immutable required content, never a persisted authorization credential.
/// The logical page uses the canonical manual contract bytes; the physical
/// page retains the original 937 plan codec. Provider-only rebinding shares
/// all three interface pages and makes only a new binding record.
#[derive(Clone, Debug)]
pub struct ForeignPages {
    pub logical: Arc<Vec<u8>>,
    pub physical: Arc<Vec<u8>>,
    pub declarations: Arc<Vec<u8>>,
    pub binding: Arc<Vec<u8>>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DeclarationPage {
    schema: String,
    declarations: Vec<SourceDecl>,
    digests: BTreeMap<String, String>,
}
#[derive(Serialize)]
struct DeclarationPageRef<'a> {
    schema: &'static str,
    declarations: &'a [SourceDecl],
    digests: &'a BTreeMap<String, String>,
}

/// The projected JSON shape without cloning module-wide fields for each
/// function. Ordering changes bytes, but never the size being admitted here.
#[derive(Serialize)]
struct ProjectedDeclRef<'a> {
    name: &'a str,
    provider: &'a str,
    logical: LogicalName<'a>,
    content_version: &'a str,
    review_id: &'a str,
    header_digest: &'a str,
    physical_table_digest: &'a str,
    context_symbol: &'a Option<String>,
    function: &'a crate::FnFact,
    effects: ProjectedEffects<'a>,
    capabilities: &'a [String],
}
struct LogicalName<'a>(&'a str, &'a str);
impl std::fmt::Display for LogicalName<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.0, self.1)
    }
}
impl Serialize for LogicalName<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}
struct ProjectedEffects<'a>(&'a [String], &'a str);
impl Serialize for ProjectedEffects<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(None)?;
        for effect in self.0 {
            if !effect.starts_with("pure:") || effect.strip_prefix("pure:") == Some(self.1) {
                seq.serialize_element(effect)?;
            }
        }
        seq.end()
    }
}

fn projected_page_size(manifest: &ManualManifest) -> Result<usize, String> {
    let limit = crate::codec::MAX_CODEC_BYTES;
    let empty: BTreeMap<String, String> = BTreeMap::new();
    let mut size = crate::codec::json_size(
        &DeclarationPageRef {
            schema: "ace.foreign.source-declarations/1",
            declarations: &[],
            digests: &empty,
        },
        limit,
    )
    .map_err(|e| format!("foreign.budget: {e}"))?;
    for (index, entry) in manifest.declarations.iter().enumerate() {
        let declaration = ProjectedDeclRef {
            name: &entry.source_name,
            provider: &manifest.provider,
            logical: LogicalName(&manifest.facts.logical, &entry.symbol),
            content_version: &manifest.facts.content_version,
            review_id: &manifest.review_id,
            header_digest: &manifest.header_digest,
            physical_table_digest: &manifest.physical_table_digest,
            context_symbol: &entry.context_symbol,
            function: manifest
                .facts
                .function(&entry.symbol)
                .ok_or("foreign.symbol")?,
            effects: ProjectedEffects(&manifest.facts.effects, &entry.symbol),
            capabilities: &manifest.facts.capabilities,
        };
        let remaining = limit.saturating_sub(size);
        let declaration_size = crate::codec::json_size(&declaration, remaining)
            .map_err(|e| format!("foreign.budget: projected declarations: {e}"))?;
        let name_size = crate::codec::json_size(entry.source_name.as_str(), remaining)
            .map_err(|e| format!("foreign.budget: projected digest name: {e}"))?;
        // Digest map entry: JSON name, colon, and a quoted 64-byte digest.
        // Both the declaration array and digest map need one comma after #0.
        size = size
            .checked_add(declaration_size)
            .and_then(|s| s.checked_add(name_size))
            .and_then(|s| s.checked_add(67 + usize::from(index != 0) * 2))
            .filter(|s| *s <= limit)
            .ok_or("foreign.budget: projected declarations exceed codec budget")?;
    }
    Ok(size)
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct BindingPage {
    schema: String,
    contract: String,
    physical: String,
    declarations: String,
    provider: String,
    generation: u64,
}
impl ForeignPages {
    pub fn iter(&self) -> impl Iterator<Item = (&'static str, &Arc<Vec<u8>>)> {
        [
            ("foreign-logical", &self.logical),
            ("foreign-physical", &self.physical),
            ("foreign-declarations", &self.declarations),
            ("foreign-binding", &self.binding),
        ]
        .into_iter()
    }
    fn binding(&self, host: &HostAuthority) -> Result<Arc<Vec<u8>>, String> {
        crate::work::produce("foreign-binding-encode", || {
            Ok(Arc::new(
                crate::codec::encode_json_bounded(
                    &BindingPage {
                        schema: "ace.foreign.binding/1".into(),
                        contract: host.contract_digest.clone(),
                        physical: crate::blake3_hex(&self.physical),
                        declarations: crate::blake3_hex(&self.declarations),
                        provider: host.provider_digest.clone(),
                        generation: host.binding_generation,
                    },
                    crate::codec::MAX_CODEC_BYTES,
                )
                .map_err(|e| e.to_string())?,
            ))
        })
    }
}

fn ident(s: &str) -> bool {
    let mut bytes = s.bytes();
    matches!(bytes.next(), Some(b) if b.is_ascii_alphabetic() || b == b'_')
        && bytes.all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

fn digest(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}
fn manual_facts(
    functions: &[crate::FnFact],
    effects: &[String],
    capabilities: &[String],
) -> Result<(), String> {
    if functions.iter().any(|f| match &f.form {
        FnForm::Pure { params, .. } | FnForm::Fallible { params, .. } => {
            params.len() > u8::MAX as usize
        }
    }) {
        return Err(
            "foreign.signature: manual source calls support at most 255 logical arguments".into(),
        );
    }
    if !capabilities.iter().any(|c| c == SOURCE_CAP) {
        return Err(
            "foreign.capability: explicit source capability is required in the declaration".into(),
        );
    }
    if functions.iter().any(|f| f.semantic_ref.is_some()) {
        return Err("foreign.semantic-ref: this manual lane has no semantic-service authority; unresolved or persisted refs are not callable".into());
    }
    if effects
        .iter()
        .any(|e| e != "checked" && !functions.iter().any(|f| e == &format!("pure:{}", f.symbol)))
    {
        return Err(
            "foreign.effect: unknown/ownership effects are outside the manual scalar lane".into(),
        );
    }
    for f in functions {
        let required = match f.form {
            FnForm::Pure { .. } => format!("pure:{}", f.symbol),
            FnForm::Fallible { .. } => "checked".into(),
        };
        // A reviewed mixed interface can carry the module's `checked`
        // effects while each total C form remains explicitly Pure. This is
        // the existing bool/order/fallible fixture's admitted contract.
        if !effects.contains(&required)
            && !(matches!(f.form, FnForm::Pure { .. }) && effects.iter().any(|e| e == "checked"))
        {
            return Err("foreign.effect: missing explicit totality/error evidence".into());
        }
    }
    Ok(())
}

impl ManualManifest {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != MANUAL_SCHEMA {
            return Err("foreign.schema: unknown manual declaration version".into());
        }
        if self.target != crate::plan::TARGET
            || self.profile != crate::plan::PROFILE
            || self.callconv != "windows-x64"
            || self.direction != "source-import"
        {
            return Err("foreign.abi: unsupported target/profile/callconv/direction".into());
        }
        if self.review_id.is_empty()
            || self.provider.is_empty()
            || !digest(&self.header_digest)
            || !digest(&self.physical_table_digest)
        {
            return Err("foreign.source: missing reviewed provenance".into());
        }
        self.facts.validate()?;
        if !matches!(&self.facts.source,crate::FactsSource::Typed{id} if !id.is_empty()) {
            return Err("foreign.source: manual imports require an explicitly reviewed typed source, never a self-claimed checked-source credential".into());
        }
        manual_facts(
            &self.facts.functions,
            &self.facts.effects,
            &self.facts.capabilities,
        )?;
        if self.declarations.is_empty() || self.declarations.len() != self.facts.functions.len() {
            return Err(
                "foreign.selection: declarations must cover every function exactly once".into(),
            );
        }
        let mut names = BTreeSet::new();
        let mut symbols = BTreeSet::new();
        for d in &self.declarations {
            if !ident(&d.source_name)
                || !ident(&d.symbol)
                || !names.insert(&d.source_name)
                || !symbols.insert(&d.symbol)
            {
                return Err("foreign.name: invalid or duplicate name/symbol".into());
            }
            let f = self
                .facts
                .function(&d.symbol)
                .ok_or("foreign.symbol: declaration has no logical facts")?;
            let scalar = |k: &LaneKind| matches!(k, LaneKind::I32 | LaneKind::Bool);
            let supported = match &f.form {
                FnForm::Pure { params, ret } => params.iter().all(scalar) && scalar(ret),
                FnForm::Fallible { params, outs, .. } => {
                    params.iter().all(scalar) && outs.len() == 1 && outs.iter().all(scalar)
                }
            };
            if !supported {
                return Err("foreign.signature: source requires i32/bool and one result".into());
            }
            let needs_context = matches!(f.form, FnForm::Fallible { ctx: Some(_), .. });
            if needs_context != d.context_symbol.is_some()
                || d.context_symbol.as_ref().is_some_and(|s| !ident(s))
            {
                return Err("foreign.context: explicit target context symbol required exactly for ctx forms".into());
            }
        }
        Ok(())
    }

    /// Contract identity excludes provider implementation bytes. A body-only
    /// provider edit changes the link/binding root, never the caller signature.
    pub fn contract_digest(&self) -> Result<String, String> {
        Ok(crate::blake3_hex(&self.canonical_bytes()?))
    }
    fn canonical_bytes(&self) -> Result<Vec<u8>, String> {
        crate::work::observe("foreign-logical-encode", false);
        // Canonicalization only reorders fields, so this checks the exact
        // output size before cloning/sorting the owned input.
        crate::codec::json_size(self, crate::codec::MAX_CODEC_BYTES)
            .map_err(|e| format!("foreign.budget: manual manifest: {e}"))?;
        self.validate()?;
        let mut canonical = self.clone();
        canonical
            .declarations
            .sort_by(|a, b| a.source_name.cmp(&b.source_name));
        canonical.facts =
            serde_json::from_str(&self.facts.canonical()?).map_err(|e| e.to_string())?;
        let bytes = crate::codec::encode_json_bounded(&canonical, crate::codec::MAX_CODEC_BYTES)
            .map_err(|e| e.to_string())?;
        crate::work::observe("foreign-logical-encode", true);
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() > crate::codec::MAX_CODEC_BYTES {
            return Err("foreign.budget: manual manifest exceeds decode budget".into());
        }
        let manifest: Self =
            serde_json::from_slice(bytes).map_err(|e| format!("foreign.decode: {e}"))?;
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn admit(self, host: &HostAuthority) -> Result<AdmittedForeign, String> {
        let logical = Arc::new(self.canonical_bytes()?);
        let contract_digest = crate::blake3_hex(&logical);
        qualify_host(&self, &contract_digest, host)?;
        projected_page_size(&self)?;
        let plan = PhysicalPlan::project(&self.facts)?;
        crate::work::observe("foreign-source-project", false);
        let declarations = project_source_declarations(&self);
        let declaration_digests: BTreeMap<_, _> = declarations
            .iter()
            .map(|d| (d.name.clone(), d.digest()))
            .collect();
        crate::work::observe("foreign-source-project", true);
        let mut pages = ForeignPages {
            logical,
            physical: Arc::new(crate::work::produce("foreign-plan-encode", || {
                crate::codec::plan_codec::encode(&plan).map_err(|e| e.to_string())
            })?),
            declarations: Arc::new(crate::work::produce("foreign-declarations-encode", || {
                crate::codec::encode_json_bounded(
                    &DeclarationPageRef {
                        schema: "ace.foreign.source-declarations/1",
                        declarations: &declarations,
                        digests: &declaration_digests,
                    },
                    crate::codec::MAX_CODEC_BYTES,
                )
                .map_err(|e| e.to_string())
            })?),
            binding: Arc::default(),
        };
        pages.binding = pages.binding(host)?;
        Ok(AdmittedForeign {
            declarations: Arc::new(declarations),
            declaration_digests: Arc::new(declaration_digests),
            manifest: Arc::new(self),
            plan: Arc::new(plan),
            contract_digest,
            provider_digest: host.provider_digest.clone(),
            generation: host.binding_generation,
            pages,
            allocation_identity: Arc::new(()),
        })
    }
}

impl AdmittedForeign {
    pub fn source_declarations(&self) -> Vec<SourceDecl> {
        self.declarations.as_ref().clone()
    }
    pub fn pages(&self) -> &ForeignPages {
        &self.pages
    }
    pub fn allocation_identity(&self) -> &Arc<()> {
        &self.allocation_identity
    }

    /// Admit required decoded content under NEW current authority. No parser,
    /// interface extraction, physical projection or artifact encoder runs.
    /// The enclosing host reserves decode/index coexistence before this call.
    pub fn restore_pages(pages: ForeignPages, host: &HostAuthority) -> Result<Self, String> {
        if pages
            .iter()
            .any(|(_, p)| p.len() > crate::codec::MAX_CODEC_BYTES)
        {
            return Err("foreign.restore.interface-budget".into());
        }
        let contract_digest = crate::blake3_hex(&pages.logical);
        let manifest = ManualManifest::decode(&pages.logical)?;
        qualify_host(&manifest, &contract_digest, host)?;
        let plan = crate::codec::plan_codec::decode(&pages.physical).map_err(|e| e.to_string())?;
        plan.validate_against(&manifest.facts)?;
        let declared: DeclarationPage = serde_json::from_slice(&pages.declarations)
            .map_err(|e| format!("foreign.restore.declarations: {e}"))?;
        if declared.schema != "ace.foreign.source-declarations/1"
            || declared.declarations.len() != manifest.declarations.len()
            || declared.digests.len() != declared.declarations.len()
            || declared
                .declarations
                .windows(2)
                .any(|p| p[0].name >= p[1].name)
        {
            return Err("foreign.restore.declaration-coverage".into());
        }
        for declaration in &declared.declarations {
            validate_source_declaration(&manifest, declaration)?;
            if declared.digests.get(&declaration.name) != Some(&declaration.digest()) {
                return Err("foreign.restore.declaration-digest".into());
            }
        }
        let binding: BindingPage = serde_json::from_slice(&pages.binding)
            .map_err(|e| format!("foreign.restore.binding: {e}"))?;
        if binding.schema != "ace.foreign.binding/1"
            || binding.contract != contract_digest
            || binding.physical != crate::blake3_hex(&pages.physical)
            || binding.declarations != crate::blake3_hex(&pages.declarations)
            || binding.provider != host.provider_digest
            || binding.generation != host.binding_generation
        {
            return Err("foreign.restore.binding-identity".into());
        }
        Ok(Self {
            manifest: Arc::new(manifest),
            plan: Arc::new(plan),
            contract_digest,
            provider_digest: host.provider_digest.clone(),
            generation: host.binding_generation,
            declarations: Arc::new(declared.declarations),
            declaration_digests: Arc::new(declared.digests),
            pages,
            allocation_identity: Arc::new(()),
        })
    }

    /// The admitted interface is immutable; a provider body edit changes only
    /// process-local authority and its untrusted binding-dependency page.
    pub fn rebind(&self, host: &HostAuthority) -> Result<Self, String> {
        qualify_host(&self.manifest, &self.contract_digest, host)?;
        let mut result = self.clone();
        result.provider_digest = host.provider_digest.clone();
        result.generation = host.binding_generation;
        result.pages.binding = result.pages.binding(host)?;
        Ok(result)
    }

    /// Validate persisted required pages against an already current admitted
    /// environment without manufacturing authority from those pages.
    pub fn restore_required(&self, pages: ForeignPages) -> Result<Self, String> {
        for ((_, current), (_, persisted)) in self.pages.iter().zip(pages.iter()) {
            if current.as_slice() != persisted.as_slice() {
                return Err("foreign.restore.current-interface".into());
            }
        }
        // The current environment already owns deeply admitted typed data.
        // Matching byte identities reuse it; no producer or encoder is entered.
        let mut result = self.clone();
        result.pages = pages;
        Ok(result)
    }
    pub fn declaration_digest(&self, name: &str) -> Option<&str> {
        self.declaration_digests.get(name).map(String::as_str)
    }
    pub fn admits_declaration(&self, declaration: &SourceDecl) -> bool {
        self.declarations
            .binary_search_by(|d| d.name.cmp(&declaration.name))
            .ok()
            .is_some_and(|index| self.declarations[index] == *declaration)
    }
    /// Revalidate current permission/registry/binding metadata. The already
    /// admitted immutable physical plan is neither reconstructed nor encoded.
    pub fn requalify(&self, host: &HostAuthority) -> Result<(), String> {
        qualify_host(&self.manifest, &self.contract_digest, host)?;
        if host.provider_digest != self.provider_digest
            || host.binding_generation != self.generation
        {
            return Err("foreign.authority.binding-changed".into());
        }
        Ok(())
    }
    pub fn manifest(&self) -> &ManualManifest {
        &self.manifest
    }
    pub fn plan(&self) -> &PhysicalPlan {
        &self.plan
    }
    pub fn contract_digest(&self) -> &str {
        &self.contract_digest
    }
    pub fn provider_digest(&self) -> &str {
        &self.provider_digest
    }
    pub fn generation(&self) -> u64 {
        self.generation
    }
    pub fn declarations(&self) -> BTreeMap<&str, &crate::FnFact> {
        self.manifest
            .declarations
            .iter()
            .map(|d| {
                (
                    d.source_name.as_str(),
                    self.manifest
                        .facts
                        .function(&d.symbol)
                        .expect("admitted symbol"),
                )
            })
            .collect()
    }
}

fn validate_source_declaration(
    manifest: &ManualManifest,
    declaration: &SourceDecl,
) -> Result<(), String> {
    declaration.validate()?;
    let entry = manifest
        .declarations
        .iter()
        .find(|d| d.source_name == declaration.name)
        .ok_or("foreign.restore.unknown-declaration")?;
    let function = manifest
        .facts
        .function(&entry.symbol)
        .ok_or("foreign.restore.logical-symbol")?;
    let logical_symbol = declaration
        .logical
        .strip_prefix(&manifest.facts.logical)
        .and_then(|s| s.strip_prefix('/'));
    let expected_effects = manifest.facts.effects.iter().filter(|e| {
        !e.starts_with("pure:") || e.strip_prefix("pure:") == Some(entry.symbol.as_str())
    });
    let valid_set = |actual: &[String], expected: &[String]| {
        actual.len() == expected.len()
            && actual.windows(2).all(|w| w[0] <= w[1])
            && actual.iter().all(|a| {
                actual.iter().filter(|x| *x == a).count()
                    == expected.iter().filter(|x| *x == a).count()
            })
    };
    if declaration.provider != manifest.provider
        || logical_symbol != Some(entry.symbol.as_str())
        || declaration.content_version != manifest.facts.content_version
        || declaration.review_id != manifest.review_id
        || declaration.header_digest != manifest.header_digest
        || declaration.physical_table_digest != manifest.physical_table_digest
        || declaration.context_symbol != entry.context_symbol
        || &declaration.function != function
        || declaration.effects.len() != expected_effects.clone().count()
        || !declaration.effects.windows(2).all(|w| w[0] <= w[1])
        || !declaration.effects.iter().all(|e| {
            declaration.effects.iter().filter(|a| *a == e).count()
                == expected_effects.clone().filter(|a| *a == e).count()
        })
        || !valid_set(&declaration.capabilities, &manifest.facts.capabilities)
    {
        return Err("foreign.restore.declaration-logical-mismatch".into());
    }
    Ok(())
}

fn project_source_declarations(manifest: &ManualManifest) -> Vec<SourceDecl> {
    let mut declarations: Vec<_> = manifest
        .declarations
        .iter()
        .map(|d| {
            let mut effects: Vec<_> = manifest
                .facts
                .effects
                .iter()
                .filter(|e| {
                    !e.starts_with("pure:") || e.strip_prefix("pure:") == Some(d.symbol.as_str())
                })
                .cloned()
                .collect();
            effects.sort();
            let mut capabilities = manifest.facts.capabilities.clone();
            capabilities.sort();
            SourceDecl {
                name: d.source_name.clone(),
                provider: manifest.provider.clone(),
                logical: format!("{}/{}", manifest.facts.logical, d.symbol),
                content_version: manifest.facts.content_version.clone(),
                review_id: manifest.review_id.clone(),
                header_digest: manifest.header_digest.clone(),
                physical_table_digest: manifest.physical_table_digest.clone(),
                context_symbol: d.context_symbol.clone(),
                function: manifest
                    .facts
                    .function(&d.symbol)
                    .expect("admitted symbol")
                    .clone(),
                effects,
                capabilities,
            }
        })
        .collect();
    declarations.sort_by(|a, b| a.name.cmp(&b.name));
    declarations
}
fn qualify_host(
    manifest: &ManualManifest,
    contract_digest: &str,
    host: &HostAuthority,
) -> Result<(), String> {
    if !host.capabilities.contains(SOURCE_CAP)
        || manifest
            .facts
            .capabilities
            .iter()
            .any(|c| !host.capabilities.contains(c))
    {
        return Err("foreign.capability: current host denies source FFI".into());
    }
    if host.review_id != manifest.review_id
        || host.contract_digest != contract_digest
        || host.provider != manifest.provider
        || host.header_digest != manifest.header_digest
        || host.physical_table_digest != manifest.physical_table_digest
        || host.target != manifest.target
        || !digest(&host.provider_digest)
    {
        return Err("foreign.authority: current review/provider binding does not match".into());
    }
    if manifest.declarations.iter().any(|d| {
        !host.available_symbols.contains(&d.symbol)
            || d.context_symbol
                .as_ref()
                .is_some_and(|s| !host.available_symbols.contains(s))
    }) {
        return Err("foreign.symbol: provider lacks a reviewed symbol".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FactsSource, FnFact, FACTS_FORMAT};

    fn fixture() -> (ManualManifest, HostAuthority) {
        let m = ManualManifest {
            schema: MANUAL_SCHEMA.into(),
            review_id: "review/c-provider-v1".into(),
            provider: "independent-c".into(),
            header_digest: "a".repeat(64),
            physical_table_digest: "b".repeat(64),
            target: crate::plan::TARGET.into(),
            profile: crate::plan::PROFILE.into(),
            callconv: "windows-x64".into(),
            direction: "source-import".into(),
            facts: InterfaceFacts {
                format: FACTS_FORMAT.into(),
                logical: "c/provider".into(),
                content_version: "v1".into(),
                source: FactsSource::Typed {
                    id: "manual/review/c-provider-v1".into(),
                },
                functions: vec![FnFact {
                    symbol: "c_add".into(),
                    form: FnForm::Pure {
                        params: vec![LaneKind::I32, LaneKind::I32],
                        ret: LaneKind::I32,
                    },
                    error_model: None,
                    semantic_ref: None,
                }],
                effects: vec!["pure:c_add".into()],
                capabilities: vec![SOURCE_CAP.into()],
            },
            declarations: vec![ManualDecl {
                context_symbol: None,
                source_name: "foreign_add".into(),
                symbol: "c_add".into(),
            }],
        };
        let host = HostAuthority {
            review_id: m.review_id.clone(),
            contract_digest: m.contract_digest().unwrap(),
            provider: m.provider.clone(),
            provider_digest: "c".repeat(64),
            header_digest: m.header_digest.clone(),
            physical_table_digest: m.physical_table_digest.clone(),
            target: m.target.clone(),
            capabilities: [SOURCE_CAP.into()].into_iter().collect(),
            available_symbols: ["c_add".into()].into_iter().collect(),
            binding_generation: 7,
        };
        (m, host)
    }

    #[test]
    fn declaration_expansion_is_admitted_before_any_projection() {
        for oversized in [false, true] {
            let (mut manifest, mut host) = fixture();
            // Escapes exercise the JSON byte count, rather than a string-length
            // estimate. Shared fields are repeated in every projected function.
            manifest.provider = "provider\"\\\n中文".repeat(if oversized { 4_000 } else { 4 });
            manifest.facts.effects = vec!["checked".into(), "checked".into()];
            manifest.facts.capabilities.push(SOURCE_CAP.into());
            let function = manifest.facts.functions[0].clone();
            manifest.facts.functions.clear();
            manifest.declarations.clear();
            host.available_symbols.clear();
            for i in 0..32 {
                let mut function = function.clone();
                function.symbol = format!("c_function_{i}");
                host.available_symbols.insert(function.symbol.clone());
                manifest.declarations.push(ManualDecl {
                    source_name: format!("foreign_function_{i}"),
                    symbol: function.symbol.clone(),
                    context_symbol: None,
                });
                manifest.facts.functions.push(function);
            }
            host.provider = manifest.provider.clone();
            host.contract_digest = manifest.contract_digest().unwrap();
            assert!(manifest.canonical_bytes().unwrap().len() < crate::codec::MAX_CODEC_BYTES);
            if oversized {
                let rejected = crate::work::with_observer(
                    Arc::new(|stage, _| {
                        assert!(
                            stage == "foreign-logical-encode",
                            "oversized projection entered producer {stage}"
                        );
                    }),
                    || manifest.admit(&host),
                )
                .unwrap_err();
                assert!(rejected.contains("foreign.budget"), "{rejected}");
            } else {
                let expected = projected_page_size(&manifest).unwrap();
                let admitted = manifest.admit(&host).unwrap();
                assert_eq!(expected, admitted.pages.declarations.len());
                AdmittedForeign::restore_pages(admitted.pages.clone(), &host).unwrap();
            }
        }
    }

    #[test]
    fn required_interface_recovers_without_any_producer_and_rebind_shares_interface() {
        use std::sync::Mutex;
        let (manifest, host) = fixture();
        let seen = Arc::new(Mutex::new(Vec::new()));
        let captured = seen.clone();
        let environment = crate::work::with_observer(
            Arc::new(move |stage, produced| {
                captured.lock().unwrap().push((stage, produced));
            }),
            || manifest.admit(&host),
        )
        .unwrap();
        for stage in [
            "foreign-logical-encode",
            "foreign-plan-project",
            "foreign-source-project",
            "foreign-plan-encode",
            "foreign-declarations-encode",
            "foreign-binding-encode",
        ] {
            assert!(
                seen.lock().unwrap().contains(&(stage, false)),
                "{stage} missing attempt"
            );
            assert!(
                seen.lock().unwrap().contains(&(stage, true)),
                "{stage} missing output"
            );
        }
        let pages = environment.pages().clone();
        drop(environment);
        let restored = crate::work::with_observer(
            Arc::new(|stage, _| {
                panic!("recovery entered producer {stage}");
            }),
            || AdmittedForeign::restore_pages(pages.clone(), &host),
        )
        .unwrap();
        assert_eq!(
            restored.declaration_digest("foreign_add"),
            Some(restored.source_declarations()[0].digest().as_str())
        );
        let mut changed = host.clone();
        changed.provider_digest = "d".repeat(64);
        changed.binding_generation += 1;
        seen.lock().unwrap().clear();
        let captured = seen.clone();
        let rebound = crate::work::with_observer(
            Arc::new(move |stage, produced| {
                captured.lock().unwrap().push((stage, produced));
            }),
            || restored.rebind(&changed),
        )
        .unwrap();
        assert_eq!(
            *seen.lock().unwrap(),
            vec![
                ("foreign-binding-encode", false),
                ("foreign-binding-encode", true)
            ]
        );
        assert!(Arc::ptr_eq(&restored.plan, &rebound.plan));
        assert!(Arc::ptr_eq(&restored.manifest, &rebound.manifest));
        for (which, before) in pages.iter().take(3) {
            let after = rebound
                .pages()
                .iter()
                .find(|(name, _)| *name == which)
                .unwrap()
                .1;
            assert!(Arc::ptr_eq(before, after));
        }
        assert!(!Arc::ptr_eq(
            &restored.pages.binding,
            &rebound.pages.binding
        ));
        assert!(AdmittedForeign::restore_pages(rebound.pages.clone(), &changed).is_ok());
        assert!(AdmittedForeign::restore_pages(pages, &changed).is_err());
        changed.capabilities.clear();
        assert!(restored.rebind(&changed).is_err());
        assert!(AdmittedForeign::restore_pages(rebound.pages.clone(), &changed).is_err());
    }

    #[test]
    fn required_interface_rejects_rehashed_slot_declaration_and_binding_forgery() {
        let (manifest, host) = fixture();
        let environment = manifest.admit(&host).unwrap();
        for axis in 0..10 {
            let mut pages = environment.pages.clone();
            let mut plan = environment.plan.as_ref().clone();
            if axis < 6 {
                let crate::PhysForm::Pure { params, ret } = &mut plan.functions[0].form else {
                    unreachable!()
                };
                match axis {
                    0 => params[0].lane = crate::PhysLane::BoolU8,
                    1 => params[0].slot = crate::SlotClass::GpRegister { index: 1 },
                    2 => params[0].stack_bank = Some(crate::lane::SlotBank::Gp),
                    3 => params[0].name = "p1".into(),
                    4 => *ret = crate::PhysRet::RaxBoolU8,
                    5 => {
                        params.pop();
                    }
                    _ => unreachable!(),
                }
                pages.physical = Arc::new(crate::codec::plan_codec::encode(&plan).unwrap());
                pages.binding = pages.binding(&host).unwrap();
            } else if axis < 9 {
                let mut declared: DeclarationPage =
                    serde_json::from_slice(&pages.declarations).unwrap();
                match axis {
                    6 => declared.declarations[0].header_digest = "e".repeat(64),
                    7 => declared.declarations[0].effects.clear(),
                    8 => declared.declarations[0].logical = "different/c_add".into(),
                    _ => unreachable!(),
                }
                declared.digests.insert(
                    declared.declarations[0].name.clone(),
                    declared.declarations[0].digest(),
                );
                pages.declarations = Arc::new(serde_json::to_vec(&declared).unwrap());
                pages.binding = pages.binding(&host).unwrap();
            } else {
                let mut binding: BindingPage = serde_json::from_slice(&pages.binding).unwrap();
                binding.generation += 1;
                pages.binding = Arc::new(serde_json::to_vec(&binding).unwrap());
            }
            assert!(
                AdmittedForeign::restore_pages(pages, &host).is_err(),
                "axis {axis}"
            );
        }
        for stage in ["logical", "physical", "declarations", "binding"] {
            let mut pages = environment.pages.clone();
            let page = match stage {
                "logical" => &mut pages.logical,
                "physical" => &mut pages.physical,
                "declarations" => &mut pages.declarations,
                _ => &mut pages.binding,
            };
            let mut wire: serde_json::Value = serde_json::from_slice(page).unwrap();
            wire["safe"] = true.into();
            *page = Arc::new(serde_json::to_vec(&wire).unwrap());
            assert!(
                AdmittedForeign::restore_pages(pages, &host).is_err(),
                "{stage}"
            );
        }
    }

    #[test]
    fn requalification_uses_admitted_plan_and_current_registry_metadata() {
        let (m, host) = fixture();
        let environment = m.admit(&host).unwrap();
        for declaration in environment.source_declarations() {
            assert_eq!(
                environment.declaration_digest(&declaration.name),
                Some(declaration.digest().as_str())
            );
        }
        environment.requalify(&host).unwrap();
        let mut denied = host.clone();
        denied.capabilities.clear();
        assert!(environment
            .requalify(&denied)
            .unwrap_err()
            .starts_with("foreign.capability"));
        let mut changed = host.clone();
        changed.binding_generation += 1;
        assert_eq!(
            environment.requalify(&changed).unwrap_err(),
            "foreign.authority.binding-changed"
        );
        changed = host.clone();
        changed.provider_digest = "d".repeat(64);
        assert_eq!(
            environment.requalify(&changed).unwrap_err(),
            "foreign.authority.binding-changed"
        );
        changed = host.clone();
        changed.contract_digest = "e".repeat(64);
        assert!(environment.requalify(&changed).is_err());
        changed = host.clone();
        changed.available_symbols.clear();
        assert!(environment
            .requalify(&changed)
            .unwrap_err()
            .starts_with("foreign.symbol"));
    }

    #[test]
    fn source_wire_arity_is_rejected_before_physical_projection() {
        let (mut manifest, host) = fixture();
        let FnForm::Pure { params, .. } = &mut manifest.facts.functions[0].form else {
            unreachable!()
        };
        *params = vec![LaneKind::I32; 255];
        manifest.validate().unwrap(); // candidate-shape control, not a C-call authorization
        let FnForm::Pure { params, .. } = &mut manifest.facts.functions[0].form else {
            unreachable!()
        };
        params.push(LaneKind::I32);
        assert!(manifest
            .validate()
            .unwrap_err()
            .starts_with("foreign.signature"));
        let failure = crate::work::with_observer(
            Arc::new(|stage, _| {
                assert_ne!(
                    stage, "foreign-plan-project",
                    "unsupported arity reached the plan producer"
                );
            }),
            || manifest.admit(&host),
        );
        assert!(failure.unwrap_err().starts_with("foreign.signature"));
        let (manifest, host) = fixture();
        let mut declaration = manifest
            .admit(&host)
            .unwrap()
            .source_declarations()
            .remove(0);
        let FnForm::Pure { params, .. } = &mut declaration.function.form else {
            unreachable!()
        };
        *params = vec![LaneKind::I32; 256];
        assert!(declaration
            .validate()
            .unwrap_err()
            .starts_with("foreign.signature"));
    }

    #[test]
    fn current_authority_is_required_and_cannot_be_restored_from_json() {
        let (m, host) = fixture();
        let bytes = serde_json::to_vec(&m).unwrap();
        let decoded = ManualManifest::decode(&bytes).unwrap();
        assert_eq!(decoded.clone().admit(&host).unwrap().generation(), 7);
        let mut revoked = host.clone();
        revoked.capabilities.clear();
        assert!(decoded
            .admit(&revoked)
            .unwrap_err()
            .contains("foreign.capability"));
        let mut forged: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        forged["safe"] = true.into();
        assert!(ManualManifest::decode(&serde_json::to_vec(&forged).unwrap()).is_err());
    }

    #[test]
    fn provider_body_changes_binding_not_contract() {
        let (m, host) = fixture();
        let old = m.clone().admit(&host).unwrap();
        let mut edited = host.clone();
        edited.provider_digest = "d".repeat(64);
        edited.binding_generation += 1;
        let new = m.admit(&edited).unwrap();
        assert_eq!(old.contract_digest(), new.contract_digest());
        assert_eq!(old.plan(), new.plan());
        assert_ne!(old.provider_digest(), new.provider_digest());
        assert_ne!(old.generation(), new.generation());
    }

    #[test]
    fn rejects_wrong_review_provenance_target_and_missing_symbol() {
        let (m, host) = fixture();
        for axis in 0..7 {
            let mut h = host.clone();
            match axis {
                0 => h.review_id.push('x'),
                1 => h.contract_digest = "0".repeat(64),
                2 => h.header_digest = "0".repeat(64),
                3 => h.physical_table_digest = "0".repeat(64),
                4 => h.target = "other".into(),
                5 => h.provider.push('x'),
                _ => h.available_symbols.clear(),
            }
            assert!(m.clone().admit(&h).is_err(), "axis {axis}");
        }
        assert!(m.admit(&host).is_ok());
    }

    #[test]
    fn selection_and_source_type_boundary_are_not_typed_bridge_support() {
        let (m, _) = fixture();
        let mut empty = m.clone();
        empty.declarations.clear();
        assert!(empty.validate().unwrap_err().contains("foreign.selection"));
        let mut wide = m.clone();
        wide.facts.functions[0].form = FnForm::Pure {
            params: vec![LaneKind::I64],
            ret: LaneKind::Bool,
        };
        assert!(wide.validate().unwrap_err().contains("foreign.signature"));
        let mut bad_ident = m.clone();
        bad_ident.declarations[0].source_name = "3bad".into();
        assert!(bad_ident.validate().unwrap_err().contains("foreign.name"));
        assert!(m.validate().is_ok());
    }

    #[test]
    fn strict_decode_budget_version_and_contract_tampering() {
        let (m, h) = fixture();
        let mut other = m.clone();
        other.schema = "ace.foreign.manual/0".into();
        assert!(ManualManifest::decode(&serde_json::to_vec(&other).unwrap()).is_err());
        assert!(ManualManifest::decode(&vec![b' '; crate::codec::MAX_CODEC_BYTES + 1]).is_err());
        other = m;
        other.facts.functions[0].form = FnForm::Pure {
            params: vec![LaneKind::Bool, LaneKind::I32],
            ret: LaneKind::I32,
        };
        assert!(other.admit(&h).unwrap_err().contains("foreign.authority"));
    }
    #[test]
    fn manual_source_never_admits_unknown_effects_or_unqualified_semantic_services() {
        let (m, h) = fixture();
        let mut candidates = Vec::new();
        let mut x = m.clone();
        x.facts.effects.push("owns:raw-pointer".into());
        candidates.push(x);
        let mut x = m.clone();
        x.facts.effects.clear();
        candidates.push(x);
        let mut x = m.clone();
        x.facts.capabilities.clear();
        candidates.push(x);
        let mut x = m.clone();
        x.facts.functions[0].semantic_ref = Some(("wrong-service".into(), "stale-version".into()));
        candidates.push(x);
        let mut x = m.clone();
        x.facts.source = FactsSource::CheckedSource {
            module: "forged-safe".into(),
            source_sha256: "0".repeat(64),
        };
        candidates.push(x);
        for candidate in candidates {
            assert!(candidate.clone().admit(&h).is_err());
            assert!(ManualManifest::decode(&serde_json::to_vec(&candidate).unwrap()).is_err());
        }
        assert!(m.admit(&h).is_ok());
    }
}
