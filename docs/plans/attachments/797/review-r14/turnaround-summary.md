# PLAN-797 T-05 turnaround raw summary

cl discovery prewarm (untimed): 133.9 ms

abi-layout-edit GC observation: ran=false reason="below high watermark (233576 <= 67108864)" total_before=233576B total_after=233576B
fresh            n=30 ok=30 P50=395.6ms P95=542.7ms P99=594.4ms
layout-hot       n=30 ok=30 P50=246.6ms P95=318.5ms P99=334.7ms
object-hot       n=30 ok=30 P50=255.3ms P95=312.6ms P99=369.0ms
restart-disk-hot n=30 ok=30 P50=269.1ms P95=381.8ms P99=389.1ms
body-edit        n=30 ok=30 P50=273.4ms P95=307.5ms P99=388.4ms
abi-layout-edit  n=30 ok=30 P50=264.2ms P95=322.4ms P99=348.7ms
descriptor-reject n=30 ok=30 P50=0.3ms P95=0.3ms P99=0.3ms
