//! Real package/type identity and old/new generation-path consumer.
fn main() {
    let input = r#"{"profile":"development","listen":{"port":0}}"#;
    let config: std::sync::Arc<auto_lang_core::http_service_config::HttpServiceConfig> =
        std::sync::Arc::new(auto_web::contract::resolve_service_config(input, None).unwrap());
    let launch = auto_web::service_launch(config.clone());
    let from_host = launch
        .settings::<auto_web_contract::HttpServiceConfig>()
        .unwrap();
    assert!(std::sync::Arc::ptr_eq(&config, &from_host));
    assert!(auto_web_contract::resolve_service_config(r#"{"unknown":true}"#, None).is_err());
    let source = "fn main() int { 40 + 2 }";
    let (value, _) = auto_lang_core::run_with_capture(source).unwrap();
    assert_eq!(value, "42");
    let candidate = "use a2r_std::http;\nfn value() {}".to_string();
    assert_eq!(
        auto_api_gen::shared_generation::qualify_a2r_std(candidate.clone()),
        auto_generation_core::shared_generation::qualify_a2r_std(candidate)
    );
    println!("foundation: one contract type/Arc, strict config, ordinary SDK=42, generation facade compatible");
}
