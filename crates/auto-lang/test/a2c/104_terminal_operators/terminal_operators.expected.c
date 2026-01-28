#include "terminal_operators.h"

int RangeIter_Reduce(struct RangeIter *self) {
}
int RangeIter_Count(struct RangeIter *self) {
}
void RangeIter_ForEach(struct RangeIter *self) {
}

int main(void) {
    unknown range = RangeIter(1, 4);
    unknown sum = range.reduce();
    unknown count = range.count();
    return 0;
}
