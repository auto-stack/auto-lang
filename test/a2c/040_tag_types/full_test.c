#include "full_test.at.h"


int main(void) {
    struct Atom a = {.tag = ATOM_INT, .as.Int = 10};
    int v = Atom_GetValue(&a);
    return v;
}
