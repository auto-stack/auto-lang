#include "question_float.h"

struct MayFloat test_question_float(void) {
    return 3.14;
}

int main(void) {
    struct MayFloat result = test_question_float();
    return 0;
}
