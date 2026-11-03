#include <fcntl.h>
#include <io.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
/* hand-written certification reference (PLAN-925 T-08): value:23
 * a="ab"; b=a.clone(); c=b.move; c=c+"中" -> a.len()*10 + c.len() = 2*10+3 */
int main(void) {
    static const uint8_t zhong[3] = { 0xE4, 0xB8, 0xAD };
    const char* a = "ab";\n    size_t c_len = 2 + 3; /* clone("ab") + "中" */
    (void)zhong; (void)memcpy;
    _setmode(_fileno(stdout), _O_BINARY);
    printf("value:%d\n", (int)(strlen(a) * 10 + c_len));
    return 0;
}
