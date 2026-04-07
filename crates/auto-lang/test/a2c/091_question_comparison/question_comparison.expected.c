#include "question_comparison.h"

bool test_comparison(void) {
    return 42 > 10;
}

int main(void) {
    unknown result = test_comparison();
    return 0;
}
