# PLAN-797 T-05 turnaround raw summary

cl discovery prewarm (untimed): 140.6 ms

abi-layout-edit GC observation: ran=false reason="below high watermark (233576 <= 67108864)" total_before=233576B total_after=233576B
fresh            n=30 ok=30 P50=364.7ms P95=447.6ms P99=449.2ms
layout-hot       n=30 ok=30 P50=245.2ms P95=284.6ms P99=295.8ms
object-hot       n=30 ok=30 P50=261.4ms P95=336.7ms P99=384.7ms
restart-disk-hot n=30 ok=30 P50=250.4ms P95=289.3ms P99=365.6ms
body-edit        n=30 ok=30 P50=250.3ms P95=267.5ms P99=269.5ms
abi-layout-edit  n=30 ok=30 P50=246.8ms P95=263.7ms P99=363.2ms
descriptor-reject n=30 ok=30 P50=0.3ms P95=0.3ms P99=0.3ms
