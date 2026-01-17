#include "question_double.h"

struct MayDouble test_question_double(void) {
    return 2.71;
}

int main(void) {
    struct MayDouble result = test_question_double();
    return 0;
}
