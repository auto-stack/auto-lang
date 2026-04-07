#pragma once

#include <stdbool.h>
enum ConnectionKind {
    CONNECTION_CONNECTED,
    CONNECTION_DISCONNECTED,
};

struct Connection {
    enum ConnectionKind tag;
    union {
        int Connected;
        int Disconnected;
    } as;
};
bool is_first(struct Connection m);
