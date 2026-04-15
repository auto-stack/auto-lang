#include "with_constraint.h"

void* identity(void* x) {
    return x;
}

void* duplicate(void* x) {
    return x;
}

void* compare(void* a, void* b) {
    return a;
}

int main(void) {
    unknown a = identity(42);
    unknown b = duplicate(10);
    say(a);
    say(b);
    return 0;
}
