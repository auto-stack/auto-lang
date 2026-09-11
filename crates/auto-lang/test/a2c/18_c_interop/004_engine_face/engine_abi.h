#pragma once
/* auto-term engine C ABI — mirrors auto-term crates/autoterm-core/src/ffi.rs
 * (Plan 597; the engine itself ships no C header — this fixture-local copy
 * is the contract the a2c driver links against; see build-engine-face-a2c.cmd).
 * BOOL=int, DWORD=unsigned int, handles=opaque void*. */
#include <stddef.h>

extern void* autoterm_engine_spawn(int cols, int rows, const char* program);
extern void autoterm_engine_write_input(void* h, const void* bytes, size_t len);
extern int autoterm_engine_feed_ready(void* h);
extern int autoterm_engine_take_dirty_rows(void* h, int* out_rows, int cap);
extern int autoterm_engine_row_text(void* h, int row, char* out_buf, int cap);
extern int autoterm_engine_row_style(void* h, int row, unsigned int* out, int cap);
extern int autoterm_engine_cursor(void* h, int* out_row, int* out_col);
extern void autoterm_engine_resize(void* h, int cols, int rows);
extern int autoterm_engine_interrupt(void* h);
extern int autoterm_engine_is_exited(void* h);
extern void autoterm_engine_kill(void* h);
extern void autoterm_engine_free(void* h);
