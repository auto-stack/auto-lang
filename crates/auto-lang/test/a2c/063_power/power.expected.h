#pragma once

#include <stdbool.h>
enum PowerKind {
    POWER_ON,
    POWER_OFF,
};

struct Power {
    enum PowerKind tag;
    union {
        int On;
        int Off;
    } as;
};
bool is_first(struct Power m);
