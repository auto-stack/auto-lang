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

int main(void) {
    struct Atom atom = {
        .tag = ATOM_INT,
        .as.Int = 42
    };

    switch (atom.tag) {
    case ATOM_INT:
        {
            printf("%s %d\n", "Got Int:", atom.as.Int);
        }
        break;
    case ATOM_CHAR:
        {
            printf("%s %c\n", "Got Char:", atom.as.Char);
        }
        break;
    case ATOM_FLOAT:
        {
            printf("%s %f\n", "Got Float:", atom.as.Float);
        }
        break;
    }

    return 0;
}
