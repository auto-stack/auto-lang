#pragma once

enum AtomKind {
    ATOM_INT,
};

struct Atom {
    enum AtomKind tag;
    union {
        int Int;
    } as;
};
