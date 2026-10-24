Reviewer probe corrections, excluded from product findings:

1. Initial caller compared window_px with logical1000x700. Actual DPI2 reports physical2000x1400. Product positive already exited0/phase9/accepted3; corrected independent resize oracle divides physical dimensions by recorded scale. Original script is retained as native_review-initial-resize-oracle.py.txt; no product change or assertion relaxation.
2. Initial audit compared raw CRLF proposal bytes to canonical normalized SHA. Frozen proposal now uses LF matching Git object/hash and the prior receipts; SDK/locks/spec raw inputs continue exact-byte comparisons. Original audit script retained; actual canonical proposal is unchanged.
