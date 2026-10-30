//! PLAN-890 feasibility consumer. This compiles the actual single-source
//! contract without depending on Core, VM, Axum, UI or Man. It is a bounded
//! source probe; production package/type relocation is not complete yet.
pub use auto_web_contract as contract;

#[cfg(test)]
mod tests {
    use super::contract::*;

    #[test]
    fn strict_contract_keeps_identity_and_rejects_unknown_input() {
        let accepted = resolve_service_config(
            r#"{"profile":"proxy_service","allowed_hosts":["api.example.com"],"auth":{"responsibility":"edge"},"trusted_proxy_ips":["10.0.0.1"]}"#,
            None,
        )
        .unwrap();
        assert!(accepted.host_allowed(Some("api.example.com:8080")));
        assert!(!accepted.host_allowed(Some("attacker.invalid")));
        let (client, _) = accepted.effective_client_identity(
            "10.0.0.1".parse().unwrap(),
            Some("192.0.2.5"),
            Some("https"),
        );
        assert_eq!(client.client.to_string(), "192.0.2.5");
        assert!(client.via_trusted_proxy && !client.xff_rejected);
        let (rejected_chain, _) = accepted.effective_client_identity(
            "10.0.0.1".parse().unwrap(),
            Some("192.0.2.5, 198.51.100.6"),
            None,
        );
        assert!(rejected_chain.xff_rejected);
        let replay = resolve_service_config(&accepted.to_json(), None).unwrap();
        assert_eq!(
            accepted.effective_config_hash(),
            replay.effective_config_hash()
        );
        assert!(resolve_service_config(r#"{"profile":"development","unknown":1}"#, None).is_err());
        assert!(resolve_service_config(r#"{"profile":"proxy_service"}"#, None).is_err());
    }
}
