#include "tag_types.h"

int Atom_GetValue(struct Atom* self) {
{
    return 42;
}
}

int main(void) {
    unknown a = {.tag = ATOM_INT, .as.Int = 10};
    unknown v = a.get_value();
    return v;
}
