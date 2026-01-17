#pragma once

#include <stdbool.h>
enum DirectionKind {
    DIRECTION_UP,
    DIRECTION_DOWN,
};

struct Direction {
    enum DirectionKind tag;
    union {
        int Up;
        int Down;
    } as;
};
bool is_first(struct Direction m);
