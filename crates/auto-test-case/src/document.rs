//! Case document loading: pinned block tree paired with raw fence ranges.
//!
//! The vendored parser is the structural judge (heading levels, `^anchors`,
//! fence order/info); it never provides source positions, so payloads are
//! extracted from the ORIGINAL bytes by [`crate::raw_ranges`] and the two
//! views are validated against each other fence-by-fence (Plan 768
//! feasibility §2). Any divergence — count, language, or (for regular fences)
//! EOL-normalized content — is a hard error, never silently tolerated.
//!
//! EOL strategy: the tree parses an EOL-normalized copy (upstream
//! `preprocessMarkdown` mis-handles IAL lines after CRLF tables; case
//! documents must not use table IAL anyway); raw payload bytes always come
//! from the original document.

use std::ops::Range;

use autodown_test_parser::block_model::{attrGet, blockText, BlockNode, BlockType, Value};
use autodown_test_parser::markdown_parser::parse_blocks;

use crate::admission::{check_limit, AdmissionError, Limits};
use crate::raw_ranges::locate_fences;

/// Fingerprint of the vendored parser snapshot actually compiled in (Plan
/// 773 F-06 toolchain identity): one SHA-256 over the three pinned source
/// files, embedded via `include_str!` so the hash tracks the compiled bytes,
/// not whatever happens to sit next to the binary.
pub fn vendored_parser_fingerprint() -> String {
    use crate::sha256::{hex, sha256};
    let mut combined = Vec::new();
    for src in [
        include_str!("../../../vendor/autodown-test-parser/src/markdown_parser.rs"),
        include_str!("../../../vendor/autodown-test-parser/src/ial.rs"),
        include_str!("../../../vendor/autodown-test-parser/src/block_model.rs"),
    ] {
        combined.extend_from_slice(src.as_bytes());
    }
    hex(&sha256(&combined))
}

/// A validated fence of a loaded document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentFence {
    /// Trimmed info string (language tag) — identical in tree and raw views.
    pub language: String,
    /// Payload byte range in the original document bytes.
    pub payload: Range<usize>,
    /// 1-based opening fence line (in the original document).
    pub open_line: usize,
    /// 1-based closing fence line, when closed (always closed for valid docs).
    pub close_line: Option<usize>,
}

/// A top-level heading (section boundary).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeadingInfo {
    pub level: usize,
    /// `^anchor` token, when the heading carries one.
    pub anchor: Option<String>,
    /// Heading text with the anchor token stripped.
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TopBlock {
    Heading(usize), // index into headings
    Fence(usize),   // index into fences
    Other,
}

/// A loaded `.case.ad` / `.suite.ad` document.
#[derive(Debug, Clone)]
pub struct CaseDocument {
    original: Vec<u8>,
    fences: Vec<DocumentFence>,
    headings: Vec<HeadingInfo>,
    top: Vec<TopBlock>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocumentError {
    /// The document is not valid UTF-8 (payloads needing other encodings use
    /// sidecars, not the document itself).
    NotUtf8,
    /// Admission limit tripped (size).
    Resource(AdmissionError),
    /// A fence never closes — a contract error, not a payload fact.
    UnclosedFence {
        line: usize,
    },
    /// Tree and raw views disagree on fence count/language at `index`.
    Correspondence {
        index: usize,
        tree: String,
        raw: String,
    },
    /// EOL-normalized payload does not equal the tree fence text.
    ContentMismatch {
        index: usize,
        line: usize,
    },
    /// Two top-level headings declare the same anchor.
    DuplicateAnchor {
        anchor: String,
    },
    AnchorNotFound {
        anchor: String,
    },
    /// Anchor section does not hold exactly one direct fence.
    AnchorFenceCount {
        anchor: String,
        count: usize,
    },
}

impl std::fmt::Display for DocumentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DocumentError::NotUtf8 => write!(f, "document is not valid UTF-8"),
            DocumentError::Resource(e) => write!(f, "admission: {}", e),
            DocumentError::UnclosedFence { line } => write!(f, "unclosed fence at line {}", line),
            DocumentError::Correspondence { index, tree, raw } => write!(
                f,
                "fence #{}: tree language {:?} != raw info {:?}",
                index, tree, raw
            ),
            DocumentError::ContentMismatch { index, line } => write!(
                f,
                "fence #{} (line {}): tree text != EOL-normalized raw payload",
                index, line
            ),
            DocumentError::DuplicateAnchor { anchor } => {
                write!(f, "duplicate top-level anchor {:?}", anchor)
            }
            DocumentError::AnchorNotFound { anchor } => write!(f, "anchor {:?} not found", anchor),
            DocumentError::AnchorFenceCount { anchor, count } => write!(
                f,
                "anchor {:?} section holds {} direct fences (exactly one required)",
                anchor, count
            ),
        }
    }
}

impl std::error::Error for DocumentError {}

/// EOL normalization identical to upstream `normalizeNewlines`
/// (`\r\n` → `\n`, then lone `\r` → `\n`).
pub fn normalize_eols(s: &str) -> String {
    let a: String = s
        .split("\r\n")
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join("\n");
    a.split('\r')
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

impl CaseDocument {
    /// Parse with frozen default limits.
    pub fn parse(bytes: &[u8]) -> Result<Self, DocumentError> {
        Self::parse_with(bytes, &Limits::defaults())
    }

    /// Parse with explicit admission limits (runner-tuned within hard caps).
    pub fn parse_with(bytes: &[u8], limits: &Limits) -> Result<Self, DocumentError> {
        check_limit("max_document_bytes", bytes.len(), limits.max_document_bytes)
            .map_err(DocumentError::Resource)?;
        let text = std::str::from_utf8(bytes).map_err(|_| DocumentError::NotUtf8)?;

        let normalized = normalize_eols(text);
        let tree = parse_blocks(&normalized, true);
        let raw = locate_fences(bytes);

        // Tree view: fences in document order. Mermaid fences become a
        // separate upstream node that stores body WITHOUT the closing-fence
        // newline; they participate in order/language mapping only (content
        // verification skipped) — payloads always come from raw bytes.
        let mut tree_fences: Vec<(String, Option<String>)> = Vec::new();
        collect_tree_fences(&tree, &mut tree_fences);

        if tree_fences.len() != raw.len() {
            return Err(DocumentError::Correspondence {
                index: tree_fences.len().min(raw.len()),
                tree: format!("{} fences", tree_fences.len()),
                raw: format!("{} fences", raw.len()),
            });
        }

        let fences: Vec<DocumentFence> = tree_fences
            .iter()
            .zip(raw.iter())
            .enumerate()
            .map(|(i, ((lang, content), rf))| {
                if !rf.closed {
                    return Err(DocumentError::UnclosedFence { line: rf.open_line });
                }
                if lang != &rf.info {
                    return Err(DocumentError::Correspondence {
                        index: i,
                        tree: lang.clone(),
                        raw: rf.info.clone(),
                    });
                }
                if let Some(content) = content {
                    let payload = normalize_eols(
                        std::str::from_utf8(&bytes[rf.payload.clone()])
                            .map_err(|_| DocumentError::NotUtf8)?,
                    );
                    if payload != *content {
                        return Err(DocumentError::ContentMismatch {
                            index: i,
                            line: rf.open_line,
                        });
                    }
                }
                Ok(DocumentFence {
                    language: rf.info.clone(),
                    payload: rf.payload.clone(),
                    open_line: rf.open_line,
                    close_line: rf.close_line,
                })
            })
            .collect::<Result<_, _>>()?;

        // Top-level structure for section resolution.
        let mut headings = Vec::new();
        let mut top = Vec::new();
        for child in &tree.children {
            match child.kind {
                BlockType::Heading => {
                    let level = match attrGet(child.attrs.clone(), "level") {
                        Some(Value::Int(v)) => v.max(0) as usize,
                        _ => 0,
                    };
                    let anchor = match attrGet(child.attrs.clone(), "anchor") {
                        Some(Value::Str(s)) => Some(s),
                        _ => None,
                    };
                    let text = blockText(child.clone());
                    top.push(TopBlock::Heading(headings.len()));
                    headings.push(HeadingInfo {
                        level,
                        anchor,
                        text,
                    });
                }
                BlockType::Fence | BlockType::Mermaid => {
                    // Fences were collected in document order and top-level
                    // children appear in that same order.
                    let idx = top
                        .iter()
                        .filter(|b| matches!(b, TopBlock::Fence(_)))
                        .count();
                    top.push(TopBlock::Fence(idx));
                }
                _ => top.push(TopBlock::Other),
            }
        }

        // Top-level anchors must be unique (deep anchors are not referencable
        // as sections; full-document anchor uniqueness is a profile concern).
        let mut seen = std::collections::BTreeSet::new();
        for h in &headings {
            if let Some(a) = &h.anchor {
                if !seen.insert(a.clone()) {
                    return Err(DocumentError::DuplicateAnchor { anchor: a.clone() });
                }
            }
        }

        Ok(CaseDocument {
            original: bytes.to_vec(),
            fences,
            headings,
            top,
        })
    }

    /// All validated fences, in document order.
    pub fn fences(&self) -> &[DocumentFence] {
        &self.fences
    }

    /// All top-level headings, in document order.
    pub fn headings(&self) -> &[HeadingInfo] {
        &self.headings
    }

    /// Raw payload bytes of a fence (original EOLs, trailing newline before
    /// the closing fence included).
    pub fn payload_bytes(&self, fence: &DocumentFence) -> &[u8] {
        &self.original[fence.payload.clone()]
    }

    /// Resolve an anchored heading to the section's single direct fence.
    ///
    /// "Direct" means: the fence sits after the anchored heading and before
    /// the NEXT HEADING OF ANY LEVEL — a deeper sub-heading starts its own
    /// (sub)section, so fences under it are not this section's direct block.
    /// The anchored heading's own span (up to the next same-or-higher
    /// heading) may contain whole subsections; only the direct prefix fence
    /// is the anchored payload, and it must be exactly one.
    pub fn anchor_fence(&self, anchor: &str) -> Result<&DocumentFence, DocumentError> {
        let start = self
            .headings
            .iter()
            .position(|h| h.anchor.as_deref() == Some(anchor))
            .ok_or_else(|| DocumentError::AnchorNotFound {
                anchor: anchor.to_string(),
            })?;
        // `start` indexes `headings`; locate the same heading inside the
        // top-level block sequence before scanning past it.
        let top_idx = self
            .top
            .iter()
            .position(|tb| matches!(tb, TopBlock::Heading(h) if *h == start))
            .expect("every heading appears in the top-level sequence");
        let mut found: Vec<usize> = Vec::new();
        for tb in &self.top[top_idx + 1..] {
            match tb {
                TopBlock::Heading(_) => break,
                TopBlock::Fence(fi) => found.push(*fi),
                TopBlock::Other => {}
            }
        }
        if found.len() != 1 {
            return Err(DocumentError::AnchorFenceCount {
                anchor: anchor.to_string(),
                count: found.len(),
            });
        }
        Ok(&self.fences[found[0]])
    }
}

fn collect_tree_fences(b: &BlockNode, out: &mut Vec<(String, Option<String>)>) {
    match b.kind {
        BlockType::Fence => {
            let lang = match attrGet(b.attrs.clone(), "language") {
                Some(Value::Str(s)) => s,
                _ => String::new(),
            };
            out.push((lang, Some(blockText(b.clone()))));
        }
        BlockType::Mermaid => out.push(("mermaid".to_string(), None)),
        _ => {}
    }
    for child in &b.children {
        collect_tree_fences(child, out);
    }
}
