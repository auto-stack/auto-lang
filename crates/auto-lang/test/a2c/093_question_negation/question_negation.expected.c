#include "question_negation.h"

int test_negation(void) {
    return -42;
}

int main(void) {
    unknown result = test_negation();
    return 0;
}
