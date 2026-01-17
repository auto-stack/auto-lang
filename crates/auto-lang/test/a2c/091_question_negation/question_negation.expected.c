#include "question_negation.h"

struct MayInt test_negation(void) {
    return -42;
}

int main(void) {
    struct MayInt result = test_negation();
    return 0;
}
