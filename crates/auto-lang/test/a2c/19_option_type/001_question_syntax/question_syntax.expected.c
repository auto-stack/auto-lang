#include "question_syntax.h"

int test_question_int(void) {
    return 42;
}

char* test_question_str(void) {
    return "hello";
}

bool test_question_bool(void) {
    return true;
}

int main(void) {
    unknown x = test_question_int();
    unknown y = test_question_str();
    unknown z = test_question_bool();
    return 0;
}
