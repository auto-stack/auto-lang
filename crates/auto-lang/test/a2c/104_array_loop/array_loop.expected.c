int arr[4] = {1, 2, 3, 4};
int main(void) {
    int sum = 0;
    int i = 0;
    while (1) {
        if (i >= 4) {
            break;
        }
        sum = sum + arr[i];
        i = i + 1;
    }
    return sum;
}
