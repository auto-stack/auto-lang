# PLAN-797 T-05 turnaround raw summary

cl discovery prewarm (untimed): 160.6 ms

abi-layout-edit GC observation: ran=false reason="below high watermark (233576 <= 67108864)" total_before=233576B total_after=233576B
fresh            n=30 ok=30 P50=361.9ms P95=438.8ms P99=455.2ms
layout-hot       n=30 ok=30 P50=249.1ms P95=286.2ms P99=293.2ms
object-hot       n=30 ok=30 P50=270.0ms P95=339.2ms P99=350.5ms
restart-disk-hot n=30 ok=30 P50=263.1ms P95=328.5ms P99=332.9ms
body-edit        n=30 ok=30 P50=271.2ms P95=355.2ms P99=364.2ms
abi-layout-edit  n=30 ok=30 P50=319.3ms P95=489.8ms P99=708.4ms
descriptor-reject n=30 ok=30 P50=0.3ms P95=0.6ms P99=1.2ms
