#pragma once

#include <stdbool.h>
enum SourceKind {
    SOURCE_INTERNAL,
    SOURCE_EXTERNAL,
};

struct Source {
    enum SourceKind tag;
    union {
        int Internal;
        int External;
    } as;
};
bool is_first(struct Source m);
