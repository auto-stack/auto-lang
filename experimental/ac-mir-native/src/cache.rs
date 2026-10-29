//! A09'/A10' unit cache over the 783 ManagedStore (contract-decision §9):
//! independent producer namespace, LayoutData carrier only, payloads ≤
//! 256 KiB per store put (paged), envelope admission on restore — the
//! claimed identity must match the recomputed one, the content hash must
//! bind the exact bytes, and the bytes must BE a relocatable COFF. A hit
//! never re-trusts a stored "verified" flag; it re-admits from the bytes.

use crate::config::*;
use crate::fingerprint;
use crate::symbols;
use crate::MnError;
use ac_artifact_probe::codec::ChunkKind;
use ac_artifact_probe::identity::QueryKey;
use ac_artifact_probe::store::{ManagedStore, PutRequest};
use auto_av_core::mir::{MirModule, OwnerScope};

/// Content hash used for artifact byte binding (blake3, hex — same
/// primitive family the 798 native admission uses).
pub fn content_hash(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

/// The identity a COFF unit artifact claims (and restore re-checks).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoffMeta {
    pub owner: OwnerScope,
    pub unit: String,
    pub symbol: String,
    pub fp: String,
    pub abi: String,
    pub target: String,
    pub flags: String,
    pub backend: String,
    pub runtime: String,
    pub mir_schema: String,
    pub mir_semantics: String,
}

impl CoffMeta {
    pub fn for_unit(unit: &MirModule, fp: String) -> Result<Self, MnError> {
        let f = &unit.funcs[0];
        Ok(CoffMeta {
            owner: unit.owner.clone(),
            unit: f.name.clone(),
            symbol: symbols::function_symbol(&unit.owner, &f.name),
            fp,
            abi: MN_ABI.to_string(),
            target: MN_TARGET.to_string(),
            flags: MN_FLAGS.to_string(),
            backend: MN_BACKEND_TAG.to_string(),
            runtime: MN_RUNTIME_TAG.to_string(),
            mir_schema: MIR_SCHEMA_AXIS.to_string(),
            mir_semantics: MIR_SEMANTICS_AXIS.to_string(),
        })
    }
}

/// A09' lookup key of one unit (all contract §9.2 axes).
pub fn coff_unit_key(meta: &CoffMeta) -> QueryKey {
    QueryKey::new(
        &COFF_LAYER,
        vec![
            ("producer".into(), PRODUCER_ID.into()),
            (
                "owner".into(),
                format!("{}|{}", meta.owner.schema, meta.owner.module),
            ),
            ("unit".into(), meta.unit.clone()),
            ("fp".into(), meta.fp.clone()),
            ("abi".into(), meta.abi.clone()),
            ("target".into(), meta.target.clone()),
            ("flags".into(), meta.flags.clone()),
            ("backend".into(), meta.backend.clone()),
            ("runtime".into(), meta.runtime.clone()),
            ("mir-schema".into(), meta.mir_schema.clone()),
            ("mir-semantics".into(), meta.mir_semantics.clone()),
        ],
    )
}

fn page_key(base: &QueryKey, page: u32) -> QueryKey {
    let mut comps = base.components.clone();
    comps.push(("page".into(), page.to_string()));
    QueryKey::new(&COFF_LAYER, comps)
}

fn esc(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('|', "\\p")
}

fn unesc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('p') => out.push('|'),
                Some('\\') => out.push('\\'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn header_lines(meta: &CoffMeta, sha: &str, len: usize, pages: usize) -> String {
    format!(
        "mn-coff 1\nschema {}\nproducer {}\nowner {}|{}\nunit {}\nsymbol {}\nfp {}\nabi {}\ntarget {}\nflags {}\nbackend {}\nruntime {}\nmir-schema {}\nmir-semantics {}\nsha256 {}\nlen {}\npages {}\n",
        COFF_SCHEMA,
        PRODUCER_ID,
        esc(&meta.owner.schema),
        esc(&meta.owner.module),
        esc(&meta.unit),
        esc(&meta.symbol),
        meta.fp,
        meta.abi,
        meta.target,
        meta.flags,
        meta.backend,
        meta.runtime,
        meta.mir_schema,
        meta.mir_semantics,
        sha,
        len,
        pages,
    )
}

fn parse_header(bytes: &[u8]) -> Result<(Vec<(String, String)>, usize), MnError> {
    const TAG: &[u8] = b"mn-coff 1\n";
    let Some(rest) = bytes.strip_prefix(TAG) else {
        return Err(MnError::Admit(
            "envelope",
            "payload is not an mn-coff/1 envelope".to_string(),
        ));
    };
    // The header block ends at the first empty line ("\n\n").
    let mut header_end = None;
    for i in 0..rest.len().saturating_sub(1) {
        if rest[i] == b'\n' && rest[i + 1] == b'\n' {
            header_end = Some(i + 2);
            break;
        }
    }
    let header_end = header_end
        .ok_or_else(|| MnError::Admit("envelope", "mn-coff header is unterminated".to_string()))?;
    let text = std::str::from_utf8(&rest[..header_end])
        .map_err(|_| MnError::Admit("envelope", "mn-coff header is not UTF-8".to_string()))?;
    let mut fields = Vec::new();
    for line in text.lines() {
        if line.is_empty() {
            continue;
        }
        let Some((k, v)) = line.split_once(' ') else {
            return Err(MnError::Admit(
                "envelope",
                format!("malformed mn-coff header line `{line}`"),
            ));
        };
        fields.push((k.to_string(), v.to_string()));
    }
    Ok((fields, header_end + TAG.len()))
}

fn field<'a>(fields: &'a [(String, String)], k: &str) -> Result<&'a str, MnError> {
    fields
        .iter()
        .find(|(fk, _)| fk == k)
        .map(|(_, v)| v.as_str())
        .ok_or_else(|| MnError::Admit("envelope", format!("missing `{k}`")))
}

/// Persist one unit's COFF (paged: every store put carries ≤ `PAGE_BYTES`
/// of payload). Returns `(page-0 artifact id, extra page ids)` — the
/// manifest references page 0 and carries the extras for restore.
pub fn put_unit_coff(
    store: &ManagedStore,
    project: Option<&str>,
    meta: &CoffMeta,
    coff: &[u8],
) -> Result<(String, Vec<String>), MnError> {
    if coff.len() > MAX_COFF_BYTES {
        return Err(MnError::Resource(
            "object-size",
            format!("COFF for `{}` exceeds the frozen cap", meta.unit),
        ));
    }
    let sha = content_hash(coff);
    let pages = coff.len().div_ceil(PAGE_BYTES).max(1);
    let key = coff_unit_key(meta);
    let header = header_lines(meta, &sha, coff.len(), pages);
    let mut first_id = None;
    let mut extra_ids = Vec::new();
    for i in 0..pages {
        let chunk = &coff[i * PAGE_BYTES..((i + 1) * PAGE_BYTES).min(coff.len())];
        let payload = if i == 0 {
            let mut p = header.clone().into_bytes();
            p.push(b'\n'); // empty line: header/body separator
            p.extend_from_slice(chunk);
            p
        } else {
            chunk.to_vec()
        };
        // Page 0 lives under the base key; further pages under derived
        // keys — the restore path recomputes both from the meta.
        let k = if i == 0 {
            key.clone()
        } else {
            page_key(&key, i as u32)
        };
        let art = store
            .put_artifact(PutRequest {
                layer: COFF_LAYER.id,
                key: k,
                kind: ChunkKind::LayoutData,
                payload,
                edges: vec![],
                project: project.map(|s| s.to_string()),
            })
            .map_err(|e| MnError::Infra(format!("store put: {e}")))?;
        if i == 0 {
            first_id = Some(art);
        } else {
            extra_ids.push(art);
        }
    }
    Ok((first_id.expect("pages >= 1"), extra_ids))
}

/// Restore one unit's COFF by its base key. `None` on a clean miss; a
/// present-but-wrong artifact is a controlled admission rejection.
pub fn get_unit_coff(store: &ManagedStore, meta: &CoffMeta) -> Result<Option<Vec<u8>>, MnError> {
    Ok(get_unit_coff_with_id(store, meta)?.map(|(_, bytes)| bytes))
}

/// Return the identity from the same admitted read as the bytes. Consumers
/// must not reopen a hit merely to recover its identity (and risk binding a
/// different concurrent lookup to these bytes).
pub fn get_unit_coff_with_id(
    store: &ManagedStore,
    meta: &CoffMeta,
) -> Result<Option<(String, Vec<u8>)>, MnError> {
    let key = coff_unit_key(meta);
    let Some(first) = store
        .get_artifact(COFF_LAYER.id, &key)
        .map_err(|e| MnError::Infra(format!("store get: {e}")))?
    else {
        return Ok(None);
    };
    let bytes = admit_payload(meta, &first.payload, &key, store)?;
    Ok(Some((first.artifact_id, bytes)))
}

/// Restore by artifact id (manifest path) with the same admission.
pub fn get_unit_coff_by_id(
    store: &ManagedStore,
    meta: &CoffMeta,
    artifact_id: &str,
) -> Result<Vec<u8>, MnError> {
    let Some(art) = store
        .get_artifact_by_id(artifact_id)
        .map_err(|e| MnError::Infra(format!("store get-by-id: {e}")))?
    else {
        return Err(MnError::Admit(
            "address",
            format!("manifest references missing artifact {artifact_id}"),
        ));
    };
    let key = coff_unit_key(meta);
    admit_payload(meta, &art.payload, &key, store)
}

fn admit_payload(
    meta: &CoffMeta,
    payload: &[u8],
    key: &QueryKey,
    store: &ManagedStore,
) -> Result<Vec<u8>, MnError> {
    let (fields, body_start) = parse_header(payload)?;
    let expect = |k: &'static str, want: &str| -> Result<(), MnError> {
        let got = field(&fields, k)?;
        if got != want {
            return Err(MnError::Admit(
                k,
                format!("artifact claims `{k}={got}`, expected `{want}`"),
            ));
        }
        Ok(())
    };
    expect("schema", COFF_SCHEMA)?;
    expect("producer", PRODUCER_ID)?;
    expect(
        "owner",
        &format!("{}|{}", esc(&meta.owner.schema), esc(&meta.owner.module)),
    )?;
    expect("unit", &esc(&meta.unit))?;
    expect("symbol", &esc(&meta.symbol))?;
    expect("fp", &meta.fp)?;
    expect("abi", &meta.abi)?;
    expect("target", &meta.target)?;
    expect("flags", &meta.flags)?;
    expect("backend", &meta.backend)?;
    expect("runtime", &meta.runtime)?;
    expect("mir-schema", &meta.mir_schema)?;
    expect("mir-semantics", &meta.mir_semantics)?;
    // The symbol claim must equal the recomputed logical symbol — a
    // self-consistent wrong-address artifact still fails here.
    let recomputed = symbols::function_symbol(&meta.owner, &meta.unit);
    if recomputed != meta.symbol {
        return Err(MnError::Admit(
            "address",
            format!(
                "meta symbol `{}` is not the logical symbol `{recomputed}`",
                meta.symbol
            ),
        ));
    }
    let pages: usize = field(&fields, "pages")?
        .parse()
        .map_err(|_| MnError::Admit("envelope", "bad pages count".into()))?;
    let len: usize = field(&fields, "len")?
        .parse()
        .map_err(|_| MnError::Admit("envelope", "bad len".into()))?;
    let sha = field(&fields, "sha256")?.to_string();
    let mut body = payload[body_start..].to_vec();
    for i in 1..pages {
        let pk = page_key(key, i as u32);
        let Some(page) = store
            .get_artifact(COFF_LAYER.id, &pk)
            .map_err(|e| MnError::Infra(format!("store get page {i}: {e}")))?
        else {
            return Err(MnError::Admit(
                "address",
                format!("page {i} of `{}` is missing", meta.unit),
            ));
        };
        body.extend_from_slice(&page.payload);
    }
    if body.len() != len {
        return Err(MnError::Admit(
            "len",
            format!(
                "restored object is {} bytes, header claims {len}",
                body.len()
            ),
        ));
    }
    let actual = content_hash(&body);
    if actual != sha {
        return Err(MnError::Admit(
            "sha256",
            format!("restored object hashes to {actual}, header claims {sha}"),
        ));
    }
    // Structural relocatable-COFF admission (798-delivered checker).
    auto_av_probe::coff_admission::check_coff_relocatable(&body)
        .map_err(|e| MnError::Admit("coff", e.to_string()))?;
    Ok(body)
}

// ---- A10' link manifest ---------------------------------------------------

#[derive(Clone, Debug)]
pub struct LinkUnitEntry {
    pub meta: CoffMeta,
    pub sha256: String,
    pub len: usize,
    pub artifact_id: String,
    pub extra_page_ids: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct LinkManifest {
    pub owner: OwnerScope,
    pub entry: String,
    pub entry_symbol: String,
    pub abi: String,
    pub target: String,
    pub units: Vec<LinkUnitEntry>,
    pub lld_sha: String,
    pub sdk_tag: String,
    pub kernel32_sha: String,
    pub trace_lib_sha: Option<String>,
    /// PLAN-870: the text runtime library's file identity — present ONLY
    /// when the link carried it (conditional axis: scalar/data manifests
    /// render and key byte-identically to the pre-870 surface).
    pub text_lib_sha: Option<String>,
}

impl LinkManifest {
    pub fn render(&self) -> String {
        let mut s = format!(
            "mn-link 1\nschema {}\nproducer {}\nowner {}|{}\nentry {}\nentry-symbol {}\nabi {}\ntarget {}\nunits {}\n",
            LINK_SCHEMA,
            PRODUCER_ID,
            esc(&self.owner.schema),
            esc(&self.owner.module),
            esc(&self.entry),
            esc(&self.entry_symbol),
            self.abi,
            self.target,
            self.units.len(),
        );
        let mut units = self.units.clone();
        units.sort_by(|a, b| a.meta.unit.cmp(&b.meta.unit));
        for u in &units {
            s.push_str(&format!(
                "unit {} {} {} {} {} {}\n",
                esc(&u.meta.unit),
                esc(&u.meta.symbol),
                u.meta.fp,
                u.sha256,
                u.len,
                u.artifact_id
            ));
            for (i, pid) in u.extra_page_ids.iter().enumerate() {
                s.push_str(&format!("upage {} {} {}\n", esc(&u.meta.unit), i + 1, pid));
            }
        }
        s.push_str(&format!(
            "link lld {}\nlink sdk {}\nlink kernel32 {}\nlink tracelib {}\n",
            self.lld_sha,
            esc(&self.sdk_tag),
            self.kernel32_sha,
            self.trace_lib_sha.clone().unwrap_or_else(|| "none".into())
        ));
        s
    }

    pub fn fingerprint(&self) -> String {
        format!(
            "{:016x}",
            auto_av_core::contract::fnv1a64(self.render().as_bytes())
        )
    }

    pub fn key(&self) -> QueryKey {
        let mut units = self.units.clone();
        units.sort_by(|a, b| a.meta.unit.cmp(&b.meta.unit));
        let mut key = QueryKey::new(
            &LINK_LAYER,
            vec![
                ("producer".into(), PRODUCER_ID.into()),
                (
                    "owner".into(),
                    format!("{}|{}", self.owner.schema, self.owner.module),
                ),
                ("entry".into(), self.entry.clone()),
                (
                    "unit-fps".into(),
                    units
                        .iter()
                        .map(|u| format!("{}:{}", u.meta.unit, u.meta.fp))
                        .collect::<Vec<_>>()
                        .join(","),
                ),
                (
                    "link-inputs".into(),
                    format!(
                        "{},{},{},{}",
                        self.lld_sha,
                        self.sdk_tag,
                        self.kernel32_sha,
                        self.trace_lib_sha.clone().unwrap_or_else(|| "none".into())
                    ),
                ),
                ("abi".into(), self.abi.clone()),
                ("entry-symbol".into(), self.entry_symbol.clone()),
            ],
        );
        if let Some(t) = &self.text_lib_sha {
            key.components.push(("textlib".into(), t.clone()));
        }
        key
    }
}

/// Persist a link manifest; returns its artifact id.
pub fn put_link_manifest(
    store: &ManagedStore,
    project: Option<&str>,
    man: &LinkManifest,
) -> Result<String, MnError> {
    if man.render().len() > METADATA_CAP {
        return Err(MnError::Resource(
            "metadata-size",
            format!("link manifest render is {} bytes", man.render().len()),
        ));
    }
    let edges: Vec<(String, ac_artifact_probe::identity::RefKind)> = man
        .units
        .iter()
        .flat_map(|u| {
            std::iter::once(u.artifact_id.clone())
                .chain(u.extra_page_ids.iter().cloned())
                .map(|id| (id, ac_artifact_probe::identity::RefKind::RequiredData))
        })
        .collect();
    store
        .put_artifact(PutRequest {
            layer: LINK_LAYER.id,
            key: man.key(),
            kind: ChunkKind::LayoutData,
            payload: man.render().into_bytes(),
            edges,
            project: project.map(|s| s.to_string()),
        })
        .map_err(|e| MnError::Infra(format!("store put manifest: {e}")))
}

/// Fetch a link manifest by its key; `None` on a clean miss.
pub fn get_link_manifest(
    store: &ManagedStore,
    key: &QueryKey,
) -> Result<Option<LinkManifest>, MnError> {
    let Some(art) = store
        .get_artifact(LINK_LAYER.id, key)
        .map_err(|e| MnError::Infra(format!("store get manifest: {e}")))?
    else {
        return Ok(None);
    };
    parse_link_manifest(&art.payload).map(Some)
}

/// Fetch + admit by key using the recomputed identity axes.
pub fn get_link_manifest_for(
    store: &ManagedStore,
    owner: &OwnerScope,
    entry: &str,
    unit_fps: &[(String, String)],
    link_inputs: &str,
    text_lib_sha: Option<&str>,
) -> Result<Option<LinkManifest>, MnError> {
    let mut sorted: Vec<_> = unit_fps.to_vec();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));
    let mut key = QueryKey::new(
        &LINK_LAYER,
        vec![
            ("producer".into(), PRODUCER_ID.into()),
            ("owner".into(), format!("{}|{}", owner.schema, owner.module)),
            ("entry".into(), entry.to_string()),
            (
                "unit-fps".into(),
                sorted
                    .iter()
                    .map(|(u, f)| format!("{u}:{f}"))
                    .collect::<Vec<_>>()
                    .join(","),
            ),
            ("link-inputs".into(), link_inputs.to_string()),
            ("abi".into(), MN_ABI.into()),
            ("entry-symbol".into(), START_SYMBOL.into()),
        ],
    );
    if let Some(t) = text_lib_sha {
        key.components.push(("textlib".into(), t.to_string()));
    }
    let Some(man) = get_link_manifest(store, &key)? else {
        return Ok(None);
    };
    // Admission: the stored render must bind to the recomputed axes.
    if man.owner != *owner || man.entry != entry || man.abi != MN_ABI || man.target != MN_TARGET {
        return Err(MnError::Admit(
            "manifest",
            "stored manifest identity does not match the recomputed axes".to_string(),
        ));
    }
    Ok(Some(man))
}

pub fn parse_link_manifest(bytes: &[u8]) -> Result<LinkManifest, MnError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| MnError::Admit("manifest", "manifest is not UTF-8".to_string()))?;
    let mut lines = text.lines();
    let first = lines
        .next()
        .ok_or_else(|| MnError::Admit("manifest", "empty manifest".into()))?;
    if first != "mn-link 1" {
        return Err(MnError::Admit(
            "manifest",
            format!("unknown manifest version line `{first}`"),
        ));
    }
    let mut owner_schema = String::new();
    let mut owner_module = String::new();
    let mut entry = String::new();
    let mut entry_symbol = String::new();
    let mut abi = String::new();
    let mut target = String::new();
    let mut n_units = 0usize;
    let mut lld_sha = String::new();
    let mut sdk_tag = String::new();
    let mut kernel32_sha = String::new();
    let mut trace_lib_sha = None;
    let mut text_lib_sha = None;
    let mut units: Vec<LinkUnitEntry> = Vec::new();
    let mut cur: Option<LinkUnitEntry> = None;
    for line in lines {
        let Some((k, v)) = line.split_once(' ') else {
            return Err(MnError::Admit(
                "manifest",
                format!("malformed line `{line}`"),
            ));
        };
        match k {
            "schema" => {
                if v != LINK_SCHEMA {
                    return Err(MnError::Admit("manifest", format!("schema `{v}`")));
                }
            }
            "producer" => {
                if v != PRODUCER_ID {
                    return Err(MnError::Admit("manifest", format!("producer `{v}`")));
                }
            }
            "owner" => {
                let Some((s, m)) = v.split_once('|') else {
                    return Err(MnError::Admit("manifest", "bad owner".into()));
                };
                owner_schema = unesc(s);
                owner_module = unesc(m);
            }
            "entry" => entry = unesc(v),
            "entry-symbol" => entry_symbol = unesc(v),
            "abi" => abi = v.to_string(),
            "target" => target = v.to_string(),
            "units" => {
                n_units = v
                    .parse()
                    .map_err(|_| MnError::Admit("manifest", "bad units count".into()))?
            }
            "unit" => {
                let parts: Vec<&str> = v.split(' ').collect();
                if parts.len() != 6 {
                    return Err(MnError::Admit("manifest", format!("bad unit line `{v}`")));
                }
                if let Some(prev) = cur.take() {
                    units.push(prev);
                }
                cur = Some(LinkUnitEntry {
                    meta: CoffMeta {
                        owner: OwnerScope {
                            schema: owner_schema.clone(),
                            module: owner_module.clone(),
                        },
                        unit: unesc(parts[0]),
                        symbol: unesc(parts[1]),
                        fp: parts[2].to_string(),
                        abi: abi.clone(),
                        target: target.clone(),
                        flags: MN_FLAGS.to_string(),
                        backend: MN_BACKEND_TAG.to_string(),
                        runtime: MN_RUNTIME_TAG.to_string(),
                        mir_schema: MIR_SCHEMA_AXIS.to_string(),
                        mir_semantics: MIR_SEMANTICS_AXIS.to_string(),
                    },
                    sha256: parts[3].to_string(),
                    len: parts[4]
                        .parse()
                        .map_err(|_| MnError::Admit("manifest", "bad unit len".into()))?,
                    artifact_id: parts[5].to_string(),
                    extra_page_ids: Vec::new(),
                });
            }
            "upage" => {
                let parts: Vec<&str> = v.split(' ').collect();
                if parts.len() != 3 {
                    return Err(MnError::Admit("manifest", format!("bad upage line `{v}`")));
                }
                let Some(u) = cur.as_mut() else {
                    return Err(MnError::Admit("manifest", "upage before unit".into()));
                };
                if u.meta.unit != unesc(parts[0]) {
                    return Err(MnError::Admit("manifest", "upage unit mismatch".into()));
                }
                u.extra_page_ids.push(parts[2].to_string());
            }
            "link" => {
                let Some((lk, lv)) = v.split_once(' ') else {
                    return Err(MnError::Admit("manifest", format!("bad link line `{v}`")));
                };
                match lk {
                    "lld" => lld_sha = lv.to_string(),
                    "sdk" => sdk_tag = unesc(lv),
                    "kernel32" => kernel32_sha = lv.to_string(),
                    "tracelib" => trace_lib_sha = (lv != "none").then(|| lv.to_string()),
                    "textlib" => text_lib_sha = Some(lv.to_string()),
                    other => {
                        return Err(MnError::Admit(
                            "manifest",
                            format!("unknown link input `{other}`"),
                        ))
                    }
                }
            }
            other => {
                return Err(MnError::Admit(
                    "manifest",
                    format!("unknown manifest key `{other}`"),
                ))
            }
        }
    }
    if let Some(prev) = cur.take() {
        units.push(prev);
    }
    if units.len() != n_units {
        return Err(MnError::Admit(
            "manifest",
            format!("manifest lists {n_units} units but carries {}", units.len()),
        ));
    }
    Ok(LinkManifest {
        owner: OwnerScope {
            schema: owner_schema,
            module: owner_module,
        },
        entry,
        entry_symbol,
        abi,
        target,
        units,
        lld_sha,
        sdk_tag,
        kernel32_sha,
        trace_lib_sha,
        text_lib_sha,
    })
}

/// Fingerprint helper for driver-side unit collection.
pub fn unit_fp(unit: &MirModule) -> String {
    fingerprint::unit_fingerprint(unit).expect("verified unit renders")
}
