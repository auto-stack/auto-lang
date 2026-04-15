int main(void) {
    int arr[4] = {1, 2, 3, 4};
    int sum = 0;
    int i = 0;
    while (i < 4) {
        sum = sum + arr[i];
        i = i + 1;
    }
    return sum;
}
