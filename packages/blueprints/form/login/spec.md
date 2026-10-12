+++
kind = "form"
name = "login"
palette = ["button", "checkbox", "input", "separator"]
extension_points = ["fields", "submit", "third_party", "success", "error_display"]
variants = ["minimal", "with_sso", "two_column"]

[dataSource]
attempt = "(creds) -> Session"

+++

# Intent

A credential-capture form that authenticates against a session endpoint. The
blueprint owns the form's structure, the submit/loading/error state machine, and
the field composition; the consumer supplies the actual `dataSource.attempt`
wired to their `#[api]` login endpoint.

# What this blueprint absorbs (per-app variation)

- which credential fields (email / username / phone / +2FA / captcha)
- success behavior (redirect vs inline)
- third-party / SSO providers
- validation rules and password policy
- visual density / branding

# Assembly guidance

- compose one `label` + `input` per credential field
- submit button -> `dataSource.attempt`; show `loading` on the button while pending
- surface auth failures via the `error_display` EDIT region
- mark every extension point with a `// EDIT: <point>` comment so consumers know
  where to customize without reading the whole widget

# References

- `minimal` — email + password + remember + submit (reference/minimal.at)
- `with_sso` — minimal plus a third-party provider row (reference/with_sso.at)
- `two_column` — split-panel layout: image/brand panel beside the minimal form
  (reference/two_column.at)

# Gotchas

See `gotchas.md`.
