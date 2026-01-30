#include "question_arithmetic.h"

struct MayInt test_arithmetic(void) {
    return 10 + 32;
}

int main(void) {
    struct MayInt result = test_arithmetic();
    return 0;
}
