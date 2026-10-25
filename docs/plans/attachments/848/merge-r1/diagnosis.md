# Consumer build-profile failure attribution

Frozen implementation: 6bc5abc6080b2261a43f55a54f127cd779239edb. No code fix.

The first fresh-consumer full run exited101 in
`assets_before_entry_faults_conflicts_unverified_recovery_and_relocated_fresh_process`.
Its child executable aborted with `thread 'main' has overflowed its stack`,
before the relocated content assertion. See consumer-test.log.txt, lines1932 onward.

The merge runner initially set RUST_MIN_STACK=16777216 but omitted the existing
Windows native acceptance link profile. RUST_MIN_STACK does not set the PE main
thread stack. Direct PE header inspection recorded consumer reserve1048576 and
producer reserve33554432 in stack-diagnosis-before.json.txt. The delivered846
consumer gate used RUSTFLAGS=-C link-arg=/STACK:33554432; current work/review
native artifacts use the same32MiB profile. This is an acceptance invocation
configuration error, not evidence that a runtime CPU/query budget may change.

The exact failed test passed after rebuilding the same source/SDK/locks with
that original profile (consumer-stack-diagnosis.log.txt,1pass/0fail). The complete
consumer gate is rerun as consumer-test-attempt2.log.txt. Neither test assertions,
node count, credits, CPU limits nor product implementation was edited. The first
failure remains in gates.json.txt and cannot be counted as a pass.

## Native Patrol scheduling boundary

The first producer user-source aggregate passed Move, then Patrol returned
`play phase 5: wrong-focus: no real 2D actions`. Its actual play_runtime records
state Paused and `auto-Pause: tick backlog >4`; ArrowRight was delivered while
Paused. The full output directory is preserved as user-failed-attempt1 (the
original invocation path was user). No unsupported/malformed source or new
segment callback was involved; the UserV1 route and this automatic pause policy
are unchanged from delivered846.

The same binary/source/input/threshold Patrol-only diagnostic completed
phase27/458frames with all original pixel/input assertions (user-patrol-diagnosis).
This establishes a successful unchanged-input control after the observed backlog,
not a proved external-process cause or an all-schedules real-time guarantee.
The complete13-case user aggregate is replayed once with the original oracle;
the original failing report and exit remain historical. No threshold, pause
condition, dropped assertion or implementation repair was introduced.
