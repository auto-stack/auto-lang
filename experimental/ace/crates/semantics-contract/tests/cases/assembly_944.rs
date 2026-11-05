use ace_semantics_contract::assembly::*;

fn semantic() -> SemanticAssembly {
    SemanticAssembly {
        profile: SCALAR_MODULES.into(),
        host: "explicit-finite/1".into(),
        platform: "x86_64-pc-windows-msvc".into(),
        declarations: vec![Declaration {
            module: "std".into(),
            symbol: "answer".into(),
            semantic_version: 1,
            signature: Signature {
                parameters: vec![],
                modes: vec![],
                returns: "i32".into(),
            },
            requirements: vec![],
        }],
        references: vec![],
        sources: vec![SourceInput {
            source_id: "std.at".into(),
            content_hash: "1".repeat(64),
        }],
    }
}
fn consumer() -> Consumer {
    Consumer {
        producer: AV_PRODUCER.into(),
        abi: "scalar-i32/1".into(),
        runtime: "av-runtime/1".into(),
        toolchain: "av/1".into(),
        providers: vec![ProviderInput {
            source_id: "provider.at".into(),
            content_hash: "2".repeat(64),
            owners: vec!["m3_std_answer".into()],
        }],
    }
}
#[test]
fn common_identity_is_consumer_neutral_but_physical_inputs_are_not() {
    let av = Manifest::freeze(semantic(), consumer()).unwrap();
    let mut rust = consumer();
    rust.producer = RUST_PRODUCER.into();
    rust.runtime = "rust-runtime/1".into();
    let rust = Manifest::freeze(semantic(), rust).unwrap();
    assert_eq!(av.fingerprint(), rust.fingerprint());
    assert_ne!(av.consumer_fingerprint(), rust.consumer_fingerprint());
    let mut next = consumer();
    next.providers[0].content_hash = "3".repeat(64);
    let next = Manifest::freeze(semantic(), next).unwrap();
    assert_eq!(av.fingerprint(), next.fingerprint());
    assert_ne!(
        av.provider_version("m3_std_answer"),
        next.provider_version("m3_std_answer")
    );
    assert_eq!(
        av.provider_version("unrelated"),
        next.provider_version("unrelated")
    );
    assert!(Manifest::admit(&av.encode().unwrap(), &next).is_err());
}
#[test]
fn stale_semantics_and_every_physical_identity_are_rejected_against_current_inputs() {
    let current = Manifest::freeze(semantic(), consumer()).unwrap();
    let bytes = current.encode().unwrap();
    assert_eq!(Manifest::admit(&bytes, &current).unwrap(), current);
    for field in ["host", "platform", "requirements", "signature", "version"] {
        let mut s = semantic();
        match field {
            "host" => s.host = "different-host/1".into(),
            "platform" => s.platform = "different-platform".into(),
            "requirements" => s.declarations[0].requirements = vec!["console".into()],
            "signature" => s.declarations[0].signature.returns = "bool".into(),
            _ => s.declarations[0].semantic_version += 1,
        }
        let changed = Manifest::freeze(s, consumer()).unwrap();
        assert_ne!(current.fingerprint(), changed.fingerprint(), "{field}");
        assert!(Manifest::admit(&bytes, &changed).is_err(), "{field}");
    }
    for field in ["abi", "runtime", "toolchain"] {
        let mut c = consumer();
        match field {
            "abi" => c.abi = "other".into(),
            "runtime" => c.runtime = "other".into(),
            _ => c.toolchain = "other".into(),
        }
        let changed = Manifest::freeze(semantic(), c).unwrap();
        assert_eq!(current.fingerprint(), changed.fingerprint());
        assert!(Manifest::admit(&bytes, &changed).is_err(), "{field}");
    }
}
#[test]
fn malformed_forged_or_wrong_domain_envelopes_are_not_admitted() {
    let current = Manifest::freeze(semantic(), consumer()).unwrap();
    for (path, value) in [
        (vec!["schema"], serde_json::json!("legacy/4")),
        (vec!["consumer", "producer"], serde_json::json!("legacy-vm")),
        (
            vec!["semantic", "profile"],
            serde_json::json!("full-language"),
        ),
        (vec!["fingerprint"], serde_json::json!("forged")),
        (vec!["consumer_fingerprint"], serde_json::json!("forged")),
    ] {
        let mut v = serde_json::to_value(&current).unwrap();
        let mut p = &mut v;
        for key in path {
            p = &mut p[key];
        }
        *p = value;
        assert!(Manifest::decode(&serde_json::to_vec(&v).unwrap()).is_err());
    }
    let mut s = semantic();
    s.declarations.push(s.declarations[0].clone());
    assert!(Manifest::freeze(s, consumer()).is_err());
    for id in [
        "C:/machine/std.at",
        "/machine/std.at",
        "../std.at",
        "std\\file.at",
    ] {
        let mut s = semantic();
        s.sources[0].source_id = id.into();
        assert!(Manifest::freeze(s, consumer()).is_err(), "{id}");
    }
    for hash in [
        "a".repeat(63),
        "g".repeat(64),
        "A".repeat(64),
        String::new(),
    ] {
        let mut s = semantic();
        s.sources[0].content_hash = hash;
        assert!(Manifest::freeze(s, consumer()).is_err());
    }
    let mut v = serde_json::to_value(&current).unwrap();
    v["host_pointer"] = 1.into();
    assert!(Manifest::decode(&serde_json::to_vec(&v).unwrap()).is_err());
}
