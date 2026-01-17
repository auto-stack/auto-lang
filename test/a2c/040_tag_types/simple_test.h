#pragma once

enum AtomKind {
    ATOM_INT,
    ATOM_FLOAT,
};

struct Atom {
    enum AtomKind tag;
    union {
        int Int;
        float Float;
    } as;
};
