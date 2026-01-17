#pragma once

#include <stdbool.h>
enum GateKind {
    GATE_OPEN,
    GATE_SHUT,
};

struct Gate {
    enum GateKind tag;
    union {
        int Open;
        int Shut;
    } as;
};
bool is_first(struct Gate m);
