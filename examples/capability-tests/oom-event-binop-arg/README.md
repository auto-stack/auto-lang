# oom-event-binop-arg (canary, GREEN)

Regression test for a parser OOM: a binary expression in a msg-call event arg
(`onclick: .Bump(1 + 1)`) made `parse_event_handler`'s arg loop spin forever
(parse_event_arg consumed only the first operand, the caller didn't handle the
leftover operator), allocating ~48 GiB until OOM.

Fixed by consuming binary operators within `parse_event_arg` (parser.rs).
`auto build` + `vue-tsc` green.

Note: a *separate* pre-existing issue remains — state-ref event args (`.n`)
render as `this.n` (wrong for Vue; should be `n`). That's tracked separately,
not by this canary.
