int main(void) {
    int (*)(int, int) add = closure_0;
    int result = add(5, 3);
    return 0;
}

int closure_0(int a, int b) {
    return a + b;
}
