# av-data-823

The first case uses the `closures:first-case` typed fixture through the real
worker and the independent parent oracle. The source is a required protocol
placeholder, not a claim of source-level closure support. The frozen result is
I32(29), allocations [4, 4, 0, 4, 6] and all owners are released; the
parent checks the exact free order from its own expectation table.

Hand-audited free order: source id 0 releases on creator exit. The env
owns the list as its last capture, so it releases first: the later element
id 4, the earlier element id 1, then list buffer id 2. The earlier text
capture id 3 releases last. Expected free IDs are `[0, 4, 1, 2, 3]`.
