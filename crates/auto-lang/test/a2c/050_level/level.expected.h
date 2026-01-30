#pragma once

#include <stdbool.h>
enum LevelKind {
    LEVEL_HIGH,
    LEVEL_LOW,
};

struct Level {
    enum LevelKind tag;
    union {
        int High;
        int Low;
    } as;
};
bool is_first(struct Level m);
