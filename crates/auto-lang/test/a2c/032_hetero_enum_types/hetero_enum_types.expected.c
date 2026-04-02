#include "tag_types.h"

int Atom_GetValue(struct Atom* self) {
{
    return 42;
}
}

int main(void) {
    struct Atom a = {.tag = ATOM_INT, .as.Int = 10};
    int v = Atom_GetValue(&a);
    return v;
}
