#pragma once

#include <stdbool.h>
enum SideKind {
    SIDE_LEFT,
    SIDE_RIGHT,
};

struct Side {
    enum SideKind tag;
    union {
        int Left;
        int Right;
    } as;
};
bool is_first(struct Side m);
