#pragma once

#include <stdio.h>

enum AtomKind {
    ATOM_INT,
    ATOM_CHAR,
    ATOM_FLOAT,
};

struct Atom {
    enum AtomKind tag;
    union {
        int Int;
        char Char;
        float Float;
    } as;
};
