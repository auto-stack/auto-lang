int src[3] = {1, 2, 3};
int dst[3] = {0, 0, 0};
int main(void) {
    dst[0] = src[0];
    dst[1] = src[1];
    dst[2] = src[2];
    return dst[0] + dst[1] + dst[2];
}
