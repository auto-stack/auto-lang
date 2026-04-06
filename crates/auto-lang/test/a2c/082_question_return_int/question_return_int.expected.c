#include "question_return_int.h"

struct MayInt test_question_return_int(void) {
    return 42;
}

int main(void) {
    unknown result = test_question_return_int();
    return 0;
}
