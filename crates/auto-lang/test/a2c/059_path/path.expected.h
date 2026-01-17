#pragma once

#include <stdbool.h>
enum PathKind {
    PATH_NORTH,
    PATH_SOUTH,
};

struct Path {
    enum PathKind tag;
    union {
        int North;
        int South;
    } as;
};
bool is_first(struct Path m);
