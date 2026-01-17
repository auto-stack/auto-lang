#include "question_nested_call.h"

int helper(void) {
    return 42;
}

struct MayInt test_nested_call(void) {
    return helper();
}

int main(void) {
    struct MayInt result = test_nested_call();
    return 0;
}
