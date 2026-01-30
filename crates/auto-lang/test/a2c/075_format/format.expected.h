#pragma once

#include <stdbool.h>
enum FormatKind {
    FORMAT_TEXT,
    FORMAT_BINARY,
};

struct Format {
    enum FormatKind tag;
    union {
        int Text;
        int Binary;
    } as;
};
bool is_first(struct Format m);
