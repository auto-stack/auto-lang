#include "question_syntax.h"

struct MayInt test_question_int(void) {
    return 42;
}

struct MayStr test_question_str(void) {
    return "hello";
}

struct MayBool test_question_bool(void) {
    return true;
}

int main(void) {
    struct MayInt x = test_question_int();
    struct MayStr y = test_question_str();
    struct MayBool z = test_question_bool();
    return 0;
}
