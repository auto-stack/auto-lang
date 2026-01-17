#include "question_return_bool.h"

struct MayBool test_question_return_bool(void) {
    return true;
}

int main(void) {
    struct MayBool result = test_question_return_bool();
    return 0;
}
