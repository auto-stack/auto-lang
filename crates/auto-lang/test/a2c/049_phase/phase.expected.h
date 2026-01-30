#pragma once

#include <stdbool.h>
enum PhaseKind {
    PHASE_START,
    PHASE_END,
};

struct Phase {
    enum PhaseKind tag;
    union {
        int Start;
        int End;
    } as;
};
bool is_first(struct Phase m);
