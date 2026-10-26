# AutoGraphics research quality and measurement baseline

PLAN-853 establishes comparable CPU/resource and evidence baselines for the independent experimental driver. It adds no game capability or performance improvement promise. Fixed Auto algorithms, behavior/1..5, scene4, source/asset bytes, SDK, lockfiles and existing budgets remain authoritative.

## Boundaries and observations

Use monotonic Rust Instant wall time with raw integer nanoseconds; retain existing microsecond aggregates under their own names. Read covers fixture/project construction and source read. Generate covers SourceSnapshot admission and managed assembly, excluding read and Session compilation. Host creation includes editing-preview compilation, device/renderer and asset initialization. Synchronous Run covers descriptor/build/Session compilation and configuration enqueue; existing compile_us is nested. Initialization runs from Run entry to accepted tick0. First rendered frame is initial editing Host rendering; Play tick0 and its frame sequence are separate.

Tick spans dispatch through accepted whole-frame and render return, including scheduling between pump rounds. Frame CPU spans Host.frame, including CPU submission and synchronous waits. Existing pump includes clock/callback/output reads/admission/renderer preparation/publication/UI synchronization. These intervals overlap and must not be summed as disjoint stages or equated with GPU latency.

Driver-local observation is opt-in and default off. Each fixed stage retains at most4096 scalar timing samples, with explicit limit/dropped count and rejection of unexpected drops. It retains no runtime owners, closures, VM, Descriptor, source or renderer handles; disabling clears samples. Session queue dispatch makes VM callback plus scalar-output collection an aggregate. Admission and renderer preparation are separately named nested observations, not extra additive cost. No SDK hook, budget, callback order or GPU timestamp is added.

## Comparable samples and limits

Use real Host/Play/Session workloads Move, Maze2D, Corridor3D, wide128 and deep128, with unchanged fixed neutral input. Each workload uses five fresh processes per observer mode, five neutral warmup ticks and30 accepted valid ticks per process. Preserve50 raw reports,1500 accepted warm ticks, per-frame timing/upload deltas, source identities, every accepted numeric document, actual initialization/final/Stop pixels and resource/owner snapshots. Off/on semantics, source, content and resident/mesh resources must match; warm publication upload cost matches and pending-only deltas must equal the same process's paused idle control, with raw sums explaining aggregate deltas.

Cold means fresh process, not flushed disk/GPU caches. Cargo build time is separate. Record machine/adapter/OS/toolchain/configuration and actual executable SHA. This baseline uses Windows x64 debug, RUST_MIN_STACK=16777216 and RUSTFLAGS=-C link-arg=/STACK:33554432; do not mix debug/release comparisons. Working set is externally sampled at10ms with actual timestamps; sampled maxima can miss shorter peaks. OS process peak is separately named and never substituted. Logical fixed-query storage is not total process memory.

P50/P95 use nearest rank at ceil(q*N)-1. Preserve raw sequences, per-process medians, pooled150 samples per workload/mode, cross-process range and paired off/on differences/ratios. Reject incomplete or invalid samples. Hashing/captures/external memory polling are outside individual frame timers; harness trace collection applies to both modes and remains within whole-tick wall time. Historical M1 aggregates are preserved, not retroactively supplied with new segmented observations. No60Hz guarantee or percentage improvement follows from these data.

## Identity and recovery

Measurement reports bind the actual immutable SHA-path executable, source identity, SDK, locks, configuration and inputs. Rebuilt executables do not replace a measured payload merely because source is equal. Reuse requires unchanged semantic inputs and verification of the original raw reports/statistics, not a fresh-sampling claim. Generated fixtures independently hash actual UTF8 source/API/bridge/combined regions for all five API versions; pre-namespace API credentials remain distinct from emitted region hashes. See [source contract](auto-source-driver.md) and [integration evidence](integration-contract.md).


Provenance: [PLAN-853](../../plans/archive/853-autographics-m2-baselines-and-evidence.md), [independent final R3 review](../../plans/attachments/853/review-r3/review.md), [merge receipt](../../plans/attachments/853/merge-r1/receipt.md). Source: autographics-dev at c1961665d1a90a8705994085fd4344a1d6a9a564; SDK 7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467. This delivers M2/S0 research baselines; production I2/I3 remain pending.
