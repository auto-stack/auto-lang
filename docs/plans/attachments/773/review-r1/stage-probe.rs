fn main() {
    let io = auto_lang::error::AutoError::Io("review injected I/O failure".into());
    let observed = auto_lang::test_cases::classify(&io);
    println!("io_stage={} io_code={:?} io_message={}", observed.stage.as_str(), observed.code, observed.message);
    let wrap = |e| auto_lang::error::AutoError::MultipleErrors { count: 1, plural: "".into(), errors: vec![e] };
    let nested = wrap(wrap(auto_lang::error::AutoError::Msg("auto_gate_review: parse failure".into())));
    let observed = auto_lang::test_cases::classify(&nested);
    println!("nested_stage={} nested_code={:?}", observed.stage.as_str(), observed.code);
}
