// Register custom cfg values so rustc doesn't warn about them being unexpected.
//
// `feature = "interpreter"` guards legacy converter implementations
// (convert_node_dynamic, etc.) that have been superseded by newer code.
// The feature is intentionally never defined, so the guarded code stays
// disabled — but we register it here to silence `unexpected_cfgs` warnings
// rather than leaving readers to wonder whether it's a typo.
fn main() {
    println!("cargo::rustc-check-cfg=cfg(feature, values(\"interpreter\"))");
}
