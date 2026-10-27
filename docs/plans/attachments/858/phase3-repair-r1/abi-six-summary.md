# PLAN-797 T-05 turnaround raw summary

cl discovery prewarm (untimed): 181.7 ms

abi-layout-edit GC observation: ran=false reason="below high watermark (233576 <= 67108864)" total_before=233576B total_after=233576B
fresh            n=30 ok=30 P50=341.5ms P90=486.9ms P95=500.8ms P99=511.9ms
layout-hot       n=30 ok=30 P50=211.5ms P90=232.4ms P95=254.2ms P99=265.3ms
object-hot       n=30 ok=30 P50=201.3ms P90=240.1ms P95=261.0ms P99=263.4ms
restart-disk-hot n=30 ok=30 P50=205.3ms P90=218.3ms P95=231.7ms P99=234.2ms
body-edit        n=30 ok=30 P50=228.3ms P90=249.5ms P95=338.8ms P99=371.5ms
abi-layout-edit  n=30 ok=30 P50=205.9ms P90=219.7ms P95=231.9ms P99=310.0ms
descriptor-reject n=30 ok=30 P50=0.1ms P90=0.1ms P95=0.2ms P99=0.2ms
