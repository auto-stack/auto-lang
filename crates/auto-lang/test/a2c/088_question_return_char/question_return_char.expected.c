#include "question_return_char.h"

struct MayChar test_question_return_char(void) {
    return 'a';
}

int main(void) {
    struct MayChar result = test_question_return_char();
    return 0;
}
