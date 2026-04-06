#include "question_uint.h"

struct MayUint test_question_uint(void) {
    return 42;
}

int main(void) {
    unknown result = test_question_uint();
    return 0;
}
