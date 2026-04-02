#include "hetero_enum.h"


int main(void) {
    struct Atom atom = {.tag = ATOM_INT, .as.Int = 11};

    switch (atom.tag) {
    case ATOM_INT:
            printf("%s %d\n", "Got Int:", atom.as.Int);
        }
        break;
    case ATOM_CHAR:
        {
            printf("%s %c\n", "Got Char:", atom.as.Char);
        }
        break;
    case ATOMFloat:
        {
            printf("%s %f\n" "Got Float:", atom.as.Float);
        }
        break;
    }
    return 0;
}
````
        break;
    }

    }
    return 0;
}

