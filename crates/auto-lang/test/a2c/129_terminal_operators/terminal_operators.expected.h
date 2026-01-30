#pragma once

struct RangeIter {
    int current;
    int end;
};

int RangeIter_Reduce(struct RangeIter *self);
int RangeIter_Count(struct RangeIter *self);
void RangeIter_ForEach(struct RangeIter *self);
