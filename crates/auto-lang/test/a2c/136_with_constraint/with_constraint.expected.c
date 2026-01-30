#include "with_constraint.h"

void identity(unknown x) {
    x;
}

void duplicate(unknown x) {
    x;
}

void compare(unknown a, unknown b) {
    a;
}

int main(void) {
    unknown a = identity(42);
    unknown b = duplicate(10);
    say(a);
    say(b);
    return 0;
}
