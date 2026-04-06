int main(void) {
    unknown (*)(int, int) add = closure_0;
    unknown result = add(5, 3);
    return 0;
}

int closure_0(int a, int b) {
    return a + b;
}
