#include "method_only.at.h"


int main(void) {
    struct Atom a = {.tag = ATOM_INT, .as.Int = 10};
    a.test();
    return 0;
}
