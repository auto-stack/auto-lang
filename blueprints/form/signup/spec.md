+++
kind = "form"
name = "signup"
palette = ["button", "checkbox", "input", "label", "separator", "text"]
extension_points = ["fields", "terms", "password_rules", "success", "error_display"]
variants = ["minimal"]
props = ["invite_token"]
actions = ["submit"]

[dataSource]
register = "(creds) -> Account"

acceptance = [
    "renders loading state on the submit button while register is pending",
    "surfaces failures in the error_display region, never as a dead submit",
    "every input has an id and an associated label",
    "terms checkbox gates submit when the terms region is materialized"
]

+++

# Intent

A credential-capture form that creates a new account. The blueprint owns the
form structure, the submit/loading/error state machine, and the password +
confirm pairing; the consumer supplies `dataSource.register` wired to their
`#[api]` signup endpoint and decides post-success navigation.

# Props

| prop | type | required | default | notes |
| --- | --- | --- | --- | --- |
| `invite_token` | str | no | `""` | invite / referral code prefilled from the route; rendered as a read-only row when non-empty |

# What this blueprint absorbs (per-app variation)

- which credential fields (email / username / phone) and password policy
- terms-of-service gating (the `terms` EDIT region)
- invite-code flow
- success behavior (auto sign-in vs redirect to login)

# Assembly guidance

- compose one `label` + `input` per credential field; pair password + confirm
  and validate equality before submit
- the `terms` EDIT region renders a `checkbox` + label; keep it checked =
  submit-eligible
- submit button -> `dataSource.register`; show `loading` on the button while
  pending; failures land in the `error_display` region
- emit `Submit(creds)`; the consumer's `actions.submit` decides navigation
- mark every extension point with a `// EDIT: <point>` comment

# References

- `minimal` — name + email + password + confirm + terms + submit
  (reference/minimal.at)

# Gotchas

See `gotchas.md`.
