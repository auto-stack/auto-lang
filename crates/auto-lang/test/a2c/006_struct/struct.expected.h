#pragma once

#include <stdio.h>

struct Point {
    int x;
    int y;
};
struct Circle {
    float radius;
    struct Point center;
};
