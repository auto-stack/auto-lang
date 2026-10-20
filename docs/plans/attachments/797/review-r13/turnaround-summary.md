# PLAN-797 T-05 turnaround raw summary

cl discovery prewarm (untimed): 135.2 ms

abi-layout-edit GC observation: ran=false reason="below high watermark (233576 <= 67108864)" total_before=233576B total_after=233576B
fresh            n=30 ok=30 P50=348.2ms P95=413.8ms P99=459.4ms
layout-hot       n=30 ok=30 P50=250.3ms P95=291.2ms P99=303.9ms
object-hot       n=30 ok=30 P50=250.0ms P95=318.1ms P99=321.9ms
restart-disk-hot n=30 ok=30 P50=239.8ms P95=319.6ms P99=377.9ms
body-edit        n=30 ok=30 P50=253.7ms P95=313.0ms P99=321.8ms
abi-layout-edit  n=30 ok=30 P50=266.8ms P95=496.9ms P99=656.1ms
descriptor-reject n=30 ok=30 P50=0.3ms P95=0.3ms P99=0.3ms
