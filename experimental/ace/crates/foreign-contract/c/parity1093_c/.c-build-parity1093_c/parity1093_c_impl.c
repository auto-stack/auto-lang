// PLAN-1093 canonical C provider impl unit: the REAL `<string.h>` strlen
// over the bounded (data, len) view (the CStr bridge), compiled with the
// FULL SDK include closure.
#include <stdint.h>
#include <string.h>

static char g_buf[65536];

int __c_strlen_impl(const uint8_t* data, uint64_t len) {
    if (len >= sizeof(g_buf)) {
        return -1;
    }
    memcpy(g_buf, data, (size_t)len);
    g_buf[len] = '\0';
    return (int)strlen(g_buf);
}
