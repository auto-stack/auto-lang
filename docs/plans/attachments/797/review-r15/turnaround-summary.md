# PLAN-797 T-05 turnaround raw summary

cl discovery prewarm (untimed): 150.5 ms

abi-layout-edit GC observation: ran=false reason="below high watermark (233576 <= 67108864)" total_before=233576B total_after=233576B
fresh            n=30 ok=30 P50=344.9ms P95=383.8ms P99=433.1ms
layout-hot       n=30 ok=30 P50=263.1ms P95=561.3ms P99=645.5ms
object-hot       n=30 ok=30 P50=374.8ms P95=610.7ms P99=639.2ms
restart-disk-hot n=30 ok=30 P50=176.2ms P95=188.2ms P99=195.9ms
body-edit        n=30 ok=30 P50=179.5ms P95=189.1ms P99=192.0ms
abi-layout-edit  n=30 ok=30 P50=180.5ms P95=198.7ms P99=248.2ms
descriptor-reject n=30 ok=30 P50=0.1ms P95=0.2ms P99=0.2ms
