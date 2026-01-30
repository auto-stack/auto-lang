#pragma once

#include <stdbool.h>
enum StateKind {
    STATE_OPEN,
    STATE_CLOSED,
};

struct State {
    enum StateKind tag;
    union {
        int Open;
        int Closed;
    } as;
};
bool is_first(struct State m);
