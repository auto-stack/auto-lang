// PLAN-1093 canonical C provider object (freestanding — no system
// headers): `c_strlen` forwards to the SDK-include impl unit.
#include "parity1093_c.h"

int c_strlen(const uint8_t* data, uint64_t len) {
    return __c_strlen_impl(data, len);
}
