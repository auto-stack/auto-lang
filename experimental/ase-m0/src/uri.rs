//! Strict `app://` URI parsing for the M0 supported domain (PLAN-1021
//! AC-01): scheme `app`, a bare lowercase identity authority, root route
//! `/` only, and no userinfo/port/query/fragment. Anything else is a named
//! `uri` refusal — there is no general URI navigation here.

use crate::error::{AseError, Kind, Phase};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppUri {
    /// The registered identity (URI authority), e.g. `auto.demo.a`.
    pub identity: String,
}

pub fn parse_app_uri(input: &str) -> Result<AppUri, AseError> {
    let refuse = |message: String| AseError::new(Phase::Resolve, Kind::Uri, message);
    let (scheme, rest) = match input.split_once("://") {
        Some(parts) => parts,
        None => return Err(refuse(format!("not an app:// URI: {input:?}"))),
    };
    if scheme != "app" {
        return Err(refuse(format!(
            "unsupported scheme {scheme:?} (only \"app\")"
        )));
    }
    // Authority runs to the first path/query/fragment marker.
    let authority_len = rest
        .find(|c| c == '/' || c == '?' || c == '#')
        .unwrap_or(rest.len());
    let authority = &rest[..authority_len];
    let tail = &rest[authority_len..];
    if authority.is_empty() {
        return Err(refuse("empty app identity".to_string()));
    }
    if let Some(at) = authority.find('@') {
        return Err(refuse(format!(
            "userinfo is not part of the app identity: {:?}",
            &authority[..at]
        )));
    }
    if let Some(colon) = authority.find(':') {
        return Err(refuse(format!(
            "port is not supported: {:?}",
            &authority[colon..]
        )));
    }
    if !authority
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '-')
    {
        return Err(refuse(format!(
            "identity {authority:?} must be lowercase [a-z0-9.-]"
        )));
    }
    if rest.contains('?') || rest.contains('#') {
        return Err(refuse(
            "query/fragment components are not supported".to_string(),
        ));
    }
    if tail != "/" {
        return Err(refuse(format!(
            "only the root route \"/\" is supported, got {tail:?}"
        )));
    }
    Ok(AppUri {
        identity: authority.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_the_two_registered_root_routes() {
        assert_eq!(
            parse_app_uri("app://auto.demo.a/").unwrap(),
            AppUri {
                identity: "auto.demo.a".into()
            }
        );
        assert_eq!(
            parse_app_uri("app://auto.demo.b/").unwrap(),
            AppUri {
                identity: "auto.demo.b".into()
            }
        );
    }

    #[test]
    fn refuses_malformed_and_out_of_domain_uris() {
        for bad in [
            "auto.demo.a/",
            "http://auto.demo.a/",
            "app:auto.demo.a/",
            "app:///",
            "app://Auto.Demo.A/",
            "app://user@auto.demo.a/",
            "app://auto.demo.a:8080/",
            "app://auto.demo.a",
            "app://auto.demo.a/ui",
            "app://auto.demo.a/?q=1",
            "app://auto.demo.a/#frag",
            "app://auto.demo.a//",
        ] {
            let err = parse_app_uri(bad).unwrap_err();
            assert_eq!(err.kind, Kind::Uri, "{bad}: {err}");
            assert_eq!(err.dispatches, 0, "{bad}");
        }
    }
}
