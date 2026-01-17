#pragma once

#include <stdbool.h>
enum SpeedKind {
    SPEED_FAST,
    SPEED_SLOW,
};

struct Speed {
    enum SpeedKind tag;
    union {
        int Fast;
        int Slow;
    } as;
};
bool is_first(struct Speed m);
