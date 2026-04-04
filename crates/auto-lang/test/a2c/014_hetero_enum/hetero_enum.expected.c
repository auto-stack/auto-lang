#include "hetero_enum.h"


int main(void) {
    struct Atom atom = {.tag = ATOM_INT, .as.Int = 11};

    switch (atom.tag) {
    case ATOM_INT:
        {
            int i = atom.as.Int;
            {
                printf("%s %d\n", "Got Int:", atom.as.Int);
            }
            break;
        }
    case ATOM_CHAR:
        {
            char c = atom.as.Char;
            {
                printf("%s %c\n", "Got Char:", atom.as.Char);
            }
            break;
        }
    case ATOM_FLOAT:
        {
            float f = atom.as.Float;
            {
                printf("%s %f\n", "Got Float:", atom.as.Float);
            }
            break;
        }
    }
    return 0;
}
