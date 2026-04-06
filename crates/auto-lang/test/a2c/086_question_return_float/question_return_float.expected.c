#include "question_return_float.h"

struct MayFloat test_question_return_float(void) {
    return 3.14;
}

int main(void) {
    unknown result = test_question_return_float();
    return 0;
}
