#include "question_return_uint.h"

struct MayUint test_question_return_uint(void) {
    return 42;
}

int main(void) {
    unknown result = test_question_return_uint();
    return 0;
}
