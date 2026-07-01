# n3-handler-local-vars (canary, GREEN)

Status: **GREEN** — `auto build` + `vue-tsc` pass.

Local mutable variables in handler blocks (`var i = 0; ...; i = i + 1`) now
work: the codegen emits `let i = 0` and bare local assignments (no `.value`),
while state assignments still use `.value`. The gap-enumeration N3 finding was
against an older base; master has since added this support, and this canary
pins it as a regression test.

(A workaround that still applies if you hit edge cases: use a state field
`var i int = 0` in the model and assign `.i`.)
