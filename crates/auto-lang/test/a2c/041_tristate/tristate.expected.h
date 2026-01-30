#pragma once

#include <stdbool.h>
enum TriStateKind {
    TRISTATE_ON,
    TRISTATE_OFF,
    TRISTATE_UNKNOWN,
};

struct TriState {
    enum TriStateKind tag;
    union {
        int On;
        int Off;
        int Unknown;
    } as;
};
bool is_first(struct TriState m);
