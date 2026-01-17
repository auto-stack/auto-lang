#pragma once

#include <stdbool.h>
enum FlowKind {
    FLOW_IN,
    FLOW_OUT,
};

struct Flow {
    enum FlowKind tag;
    union {
        int In;
        int Out;
    } as;
};
bool is_first(struct Flow m);
