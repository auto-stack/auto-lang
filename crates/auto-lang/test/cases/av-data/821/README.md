# av-data-821 corpus — PLAN-821 owned-profile first case

The corpus seeds are byte-identical copies of the frozen first-case
sources (contract-decision §11; the worker-side frozen fixture table
`auto_av_probe::owned_fixture::FIRST_CASE_SRC` and the probe first-case
test carry the same bytes — the `av_owned_821_recipe` tests pin the
identity by hash).

| seed | SHA-256 | role |
|---|---|---|
| src/first-case.at | 86720543aa29544881f9cc3244df9fe7310bda38f94024588e1ef4dddcfc880d | the frozen owned first case (record-returning; fixture `first-case`) |
| src/first-case-int-twin.at | a5f3cf2683ab14e77c99ae6878b1f39212fbc37fade1ec3530c87fc2bce1e7db | the int twin (same owned operations; exit 404 hand-audited) |

Hand-audited oracles (§11): Note { body: "Auto", n: 4 } (the result face's
nominal name is the type table's synthesized `t_rec_Note`); two 4-byte
allocations (ctor literal + clone); ctor buffer frees at the field
replace, the clone frees at the host release (creation-order LIFO);
live=0 after the host release; native twin exit 404.
