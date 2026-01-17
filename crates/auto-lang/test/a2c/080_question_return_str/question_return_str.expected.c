#include "question_return_str.h"

struct MayStr test_question_return_str(void) {
    return "test";
}

int main(void) {
    struct MayStr result = test_question_return_str();
    return 0;
}
