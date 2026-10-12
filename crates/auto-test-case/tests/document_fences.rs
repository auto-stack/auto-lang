//! Fence fidelity battery (V-02 core): raw bytes preserved through tree↔raw
//! correspondence, EOLs/anchors/unclosed handling.

use auto_test_case::admission::Limits;
use auto_test_case::document::{CaseDocument, DocumentError};

fn doc() -> Vec<u8> {
    concat!(
        "# Title\r\n",
        "\r\n",
        "Intro with unicode \u{1F600} and CJK \u{4E2D}\u{6587}.\r\n",
        "\r\n",
        "## Payloads ^auto-src\r\n",
        "\r\n",
        "````auto\r\n",
        "fn main() {\r\n",
        "    print(\"nested ``` inside\")\n",
        "}\r\n",
        "````\r\n",
        "\r\n",
        "## Contract\r\n",
        "\r\n",
        "```test-case\r\n",
        "case { id: \"probe.1\" }\r\n",
        "```\n",
        "\r\n",
        "## Empty\r\n",
        "\r\n",
        "```\n",
        "```\n",
        "\r\n",
        "## Spaced\r\n",
        "\r\n",
        "```  spaced-info  \n",
        "payload with trailing spaces   \n",
        "```   \n",
        "\r\n",
        "    ```not-a-fence\n",
        "\r\n",
        "not a fence: ``` inline\r\n",
    )
    .as_bytes()
    .to_vec()
}

#[test]
fn valid_document_battery() {
    let d = CaseDocument::parse(&doc()).expect("valid doc");
    let fences = d.fences();
    assert_eq!(
        fences.len(),
        4,
        "indented and mid-line backticks must not open fences"
    );

    let langs: Vec<&str> = fences.iter().map(|f| f.language.as_str()).collect();
    assert_eq!(langs, ["auto", "test-case", "", "spaced-info"]);

    // Payload bytes from the ORIGINAL document — CRLF/LF mix preserved.
    assert_eq!(
        d.payload_bytes(&fences[0]),
        b"fn main() {\r\n    print(\"nested ``` inside\")\n}\r\n".as_slice()
    );
    assert_eq!(
        d.payload_bytes(&fences[1]),
        b"case { id: \"probe.1\" }\r\n".as_slice()
    );
    assert_eq!(d.payload_bytes(&fences[2]), b"".as_slice());
    assert_eq!(
        d.payload_bytes(&fences[3]),
        b"payload with trailing spaces   \n".as_slice()
    );

    // Headings: level + stripped anchor.
    let heads = d.headings();
    assert_eq!(heads.len(), 5);
    assert_eq!((heads[0].level, heads[0].anchor.as_deref()), (1, None));
    assert_eq!(
        (
            heads[1].level,
            heads[1].anchor.as_deref(),
            heads[1].text.as_str()
        ),
        (2, Some("auto-src"), "Payloads")
    );

    // Anchor resolves to the first section's single fence with exact bytes.
    let f = d.anchor_fence("auto-src").expect("anchor resolves");
    assert_eq!(f.language, "auto");
    assert_eq!(
        d.payload_bytes(f),
        b"fn main() {\r\n    print(\"nested ``` inside\")\n}\r\n".as_slice()
    );
}

#[test]
fn unclosed_fence_is_error() {
    let err = CaseDocument::parse(b"```rust\nlet x = 1\n").unwrap_err();
    assert_eq!(err, DocumentError::UnclosedFence { line: 1 });
}

#[test]
fn non_utf8_is_error() {
    let err = CaseDocument::parse(b"# ok\n\n```x\n\xff\xfe\n```\n").unwrap_err();
    assert_eq!(err, DocumentError::NotUtf8);
}

#[test]
fn document_size_limit_is_resource_error() {
    let mut small = Limits::defaults();
    small.max_document_bytes = 16;
    let err = CaseDocument::parse_with(b"# plenty longer than sixteen\n", &small).unwrap_err();
    match err {
        DocumentError::Resource(e) => {
            let msg = e.to_string();
            assert!(msg.contains("max_document_bytes"), "{}", msg);
        }
        other => panic!("expected Resource, got {:?}", other),
    }
}

#[test]
fn anchor_section_requires_exactly_one_fence() {
    let zero = CaseDocument::parse(b"# A ^a\n\ntext only\n\n# B\n".as_slice())
        .unwrap()
        .anchor_fence("a")
        .unwrap_err();
    assert_eq!(
        zero,
        DocumentError::AnchorFenceCount {
            anchor: "a".into(),
            count: 0
        }
    );

    let two = CaseDocument::parse(
        concat!("# A ^a\n", "```x\n", "one\n", "```\n", "```y\n", "two\n", "```\n", "# B\n",)
            .as_bytes(),
    )
    .unwrap()
    .anchor_fence("a")
    .unwrap_err();
    assert_eq!(
        two,
        DocumentError::AnchorFenceCount {
            anchor: "a".into(),
            count: 2
        }
    );
}

#[test]
fn anchor_resolution_spans_deeper_sections_until_same_level() {
    let d = CaseDocument::parse(
        concat!(
            "# Top ^top\n",
            "```x\n",
            "one\n",
            "```\n",
            "## Sub ^sub\n",
            "```y\n",
            "two\n",
            "```\n",
            "# Next\n",
        )
        .as_bytes(),
    )
    .unwrap();
    assert_eq!(
        d.payload_bytes(d.anchor_fence("top").unwrap()),
        b"one\n".as_slice()
    );
    assert_eq!(
        d.payload_bytes(d.anchor_fence("sub").unwrap()),
        b"two\n".as_slice()
    );
    assert_eq!(
        d.anchor_fence("missing").unwrap_err(),
        DocumentError::AnchorNotFound {
            anchor: "missing".into()
        }
    );
}

#[test]
fn duplicate_top_level_anchor_is_error() {
    let err = CaseDocument::parse(
        concat!("# A ^x\n", "```a\n", "one\n", "```\n", "# B ^x\n", "```b\n", "two\n", "```\n",)
            .as_bytes(),
    )
    .unwrap_err();
    assert_eq!(err, DocumentError::DuplicateAnchor { anchor: "x".into() });
}

#[test]
fn lone_cr_and_crlf_payloads_preserved() {
    // Lone CR is a line terminator for the locator too (bytes between lines).
    let d = CaseDocument::parse(b"# T ^p\r\n\r\n```x\r\na\rb\n```\r\n").unwrap();
    let f = d.anchor_fence("p").unwrap();
    assert_eq!(d.payload_bytes(f), b"a\rb\n".as_slice());
}

#[test]
fn mermaid_fence_maps_without_content_check() {
    let d = CaseDocument::parse(
        concat!(
            "# T ^m\n",
            "```mermaid\n",
            "graph TD;\n",
            "  A-->B;\n",
            "```\n",
            "# Payload\n",
            "```x\n",
            "real\n",
            "```\n",
        )
        .as_bytes(),
    )
    .unwrap();
    let fences = d.fences();
    assert_eq!(fences.len(), 2);
    assert_eq!(fences[0].language, "mermaid");
    assert_eq!(fences[1].language, "x");
    let f = d.anchor_fence("m").unwrap();
    assert_eq!(f.language, "mermaid");
    assert_eq!(d.payload_bytes(f), b"graph TD;\n  A-->B;\n".as_slice());
}
