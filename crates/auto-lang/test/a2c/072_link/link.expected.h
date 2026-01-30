#pragma once

#include <stdbool.h>
enum LinkKind {
    LINK_CONNECTED,
    LINK_DISCONNECTED,
};

struct Link {
    enum LinkKind tag;
    union {
        int Connected;
        int Disconnected;
    } as;
};
bool is_first(struct Link m);
