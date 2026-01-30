#include "question_comparison.h"

struct MayBool test_comparison(void) {
    return 42 > 10;
}

int main(void) {
    struct MayBool result = test_comparison();
    return 0;
}
