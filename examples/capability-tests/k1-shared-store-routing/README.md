# k1-shared-store-routing (canary)

Two route pages share a module-level `store CounterStore` (a view-less widget:
`model` + `msg` + `on`). Incrementing on one route must persist when navigating
to the other — validating cross-route shared state (Design 18 / Plan 351).

Currently RED (store decl not yet implemented). Flips to GREEN when Plan 351
lands.
