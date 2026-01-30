#pragma once

#include <stdbool.h>
enum ZoneKind {
    ZONE_A,
    ZONE_B,
    ZONE_C,
};

struct Zone {
    enum ZoneKind tag;
    union {
        int A;
        int B;
        int C;
    } as;
};
bool is_first(struct Zone m);
