#pragma once

#include <stdbool.h>
enum SignalKind {
    SIGNAL_HIGH,
    SIGNAL_LOW,
};

struct Signal {
    enum SignalKind tag;
    union {
        int High;
        int Low;
    } as;
};
bool is_first(struct Signal m);
