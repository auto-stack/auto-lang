#include "question_return_double.h"

struct MayDouble test_question_return_double(void) {
    return 2.71;
}

int main(void) {
    struct MayDouble result = test_question_return_double();
    return 0;
}
