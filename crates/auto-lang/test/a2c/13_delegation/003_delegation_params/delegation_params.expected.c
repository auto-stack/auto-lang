#include "delegation_params.h"


int MathEngine_Add(struct MathEngine *self, int, int) {
    return a + b;
}
int MathEngine_Multiply(struct MathEngine *self, int, int) {
    return a * b;
}

int Computer_add(struct Computer *self, int a, int b) {
    return MathEngine_add(&self->engine, a, b);
}
int Computer_multiply(struct Computer *self, int a, int b) {
    return MathEngine_multiply(&self->engine, a, b);
}

int main(void) {
    struct Computer comp = {};
    unknown result1 = Computer_add(&comp, 5, 3);
    unknown result2 = Computer_multiply(&comp, 4, 7);
    printf("%d\n", result1);
    printf("%d\n", result2);
    return 0;
}
