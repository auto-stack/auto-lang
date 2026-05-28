#pragma once

#include <stdio.h>

enum ShapeKind {
    SHAPE_CIRCLE,
    SHAPE_RECT,
};

struct Shape {
    enum ShapeKind tag;
    union {
        float Circle;
        float Rect;
    } as;
};
float area(struct Shape s);
