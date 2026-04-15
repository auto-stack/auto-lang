int main(void) {
    int sum = 0;
    int product = 1;
    int i = 1;
    while (i < 5) {
        sum = sum + i;
        product = product * i;
        i = i + 1;
    }
    return sum + product;
}
