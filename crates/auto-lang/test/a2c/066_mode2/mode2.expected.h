#pragma once

#include <stdbool.h>
enum Mode2Kind {
    MODE2_AUTO,
    MODE2_MANUAL,
};

struct Mode2 {
    enum Mode2Kind tag;
    union {
        int Auto;
        int Manual;
    } as;
};
bool is_first(struct Mode2 m);
