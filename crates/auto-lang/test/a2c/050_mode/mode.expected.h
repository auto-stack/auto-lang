#pragma once

#include <stdbool.h>
enum ModeKind {
    MODE_READ,
    MODE_WRITE,
};

struct Mode {
    enum ModeKind tag;
    union {
        int Read;
        int Write;
    } as;
};
bool is_first(struct Mode m);
