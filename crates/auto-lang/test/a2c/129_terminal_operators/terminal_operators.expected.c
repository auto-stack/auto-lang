#include "terminal_operators.h"

int RangeIter_Reduce(struct RangeIter *self) {
}
int RangeIter_Count(struct RangeIter *self) {
}
void RangeIter_ForEach(struct RangeIter *self) {
}

int main(void) {
    struct RangeIter range = {.current = 1, .end = 4};
    unknown sum = RangeIter_Reduce(&range);
    unknown count = RangeIter_Count(&range);
    return 0;
}
