#pragma once

#include <stdbool.h>
enum ColorKind {
    COLOR_RED,
    COLOR_BLUE,
};

struct Color {
    enum ColorKind tag;
    union {
        int Red;
        int Blue;
    } as;
};
bool is_first(struct Color m);
