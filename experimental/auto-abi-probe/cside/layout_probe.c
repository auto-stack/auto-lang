/*
 * layout_probe.c — PLAN-797 T-02 C layout oracle program.
 *
 * Measures sizeof / __alignof / offsetof for every record in
 * autoabi_layout_v01.h with the real C compiler and reports one
 * structured line per fact on stdout (kernel32 WriteFile only — no CRT).
 * The Rust test parses this output and compares it against the Rust
 * layout engine, making the C compiler the independent oracle.
 *
 * Wire format (ASCII, one fact per line):
 *   R <name> S <size> A <align>
 *   F <record> <field> O <offset> W <bytewidth>
 *   N <record> <field> <subfield> O <offset> W <bytewidth>   (nested)
 *   END
 */

#include "autoabi_layout_v01.h"

int _fltused = 0; /* this TU holds FP constants (double) */

__declspec(dllimport) void *__stdcall GetStdHandle(unsigned long);
__declspec(dllimport) int __stdcall WriteFile(void *, const void *,
                                              unsigned long, unsigned long *,
                                              void *);
__declspec(dllimport) void __stdcall ExitProcess(unsigned int);

#define WBUFSZ 4096

static char g_buf[WBUFSZ];
static unsigned long g_len;

static void emit_raw(const char *s)
{
    while (*s && g_len + 1 < WBUFSZ) {
        g_buf[g_len++] = *s++;
    }
}

static void emit_u64(unsigned long long v)
{
    char tmp[24];
    int n = 0;
    if (v == 0) {
        tmp[n++] = '0';
    }
    while (v > 0 && n < 24) {
        tmp[n++] = (char)('0' + (v % 10));
        v /= 10;
    }
    while (n > 0 && g_len + 1 < WBUFSZ) {
        g_buf[g_len++] = tmp[--n];
    }
}

static void emit_str(const char *s) { emit_raw(s); }
static void emit_ch(char c)
{
    if (g_len + 1 < WBUFSZ) {
        g_buf[g_len++] = c;
    }
}

static void flush_line(void)
{
    void *h = GetStdHandle(0xFFFFFFF5UL /* STD_OUTPUT_HANDLE */);
    unsigned long written = 0;
    emit_ch('\n');
    WriteFile(h, g_buf, g_len, &written, 0);
    g_len = 0;
}

#define offsetof_r(T, F) ((unsigned long long)(size_t)&(((T *)0)->F))

#define EMIT_RECORD(NAME, T)                                                  \
    do {                                                                      \
        emit_str("R " NAME " S ");                                            \
        emit_u64((unsigned long long)sizeof(T));                              \
        emit_str(" A ");                                                      \
        emit_u64((unsigned long long)__alignof(T));                           \
        flush_line();                                                         \
    } while (0)

#define EMIT_FIELD(RNAME, T, F, SUB)                                          \
    do {                                                                      \
        if ((SUB)[0] == '\0') { emit_str("F " RNAME " " #F " O "); }          \
        else { emit_str("N " RNAME " " #F " " SUB " O "); }                    \
        emit_u64(offsetof_r(T, F));                                           \
        emit_str(" W ");                                                      \
        emit_u64((unsigned long long)sizeof(((T *)0)->F));                    \
        flush_line();                                                         \
    } while (0)

void p797_layout_entry(void)
{
    EMIT_RECORD("RecMix", RecMix);
    EMIT_FIELD("RecMix", RecMix, flag, "");
    EMIT_FIELD("RecMix", RecMix, count, "");
    EMIT_FIELD("RecMix", RecMix, tag, "");
    EMIT_FIELD("RecMix", RecMix, big, "");

    EMIT_RECORD("RecPad", RecPad);
    EMIT_FIELD("RecPad", RecPad, a, "");
    EMIT_FIELD("RecPad", RecPad, b, "");
    EMIT_FIELD("RecPad", RecPad, c, "");

    EMIT_RECORD("RecNested", RecNested);
    EMIT_FIELD("RecNested", RecNested, x, "");
    EMIT_FIELD("RecNested", RecNested, m, "");
    EMIT_FIELD("RecNested", RecNested, d, "");
    /* nested subfield offsets (absolute, not relative) */
    EMIT_FIELD("RecNested", RecNested, m.flag, "flag");
    EMIT_FIELD("RecNested", RecNested, m.count, "count");
    EMIT_FIELD("RecNested", RecNested, m.tag, "tag");
    EMIT_FIELD("RecNested", RecNested, m.big, "big");

    EMIT_RECORD("RecOrderB", RecOrderB);
    EMIT_FIELD("RecOrderB", RecOrderB, big, "");
    EMIT_FIELD("RecOrderB", RecOrderB, flag, "");
    EMIT_FIELD("RecOrderB", RecOrderB, count, "");
    EMIT_FIELD("RecOrderB", RecOrderB, tag, "");

    emit_str("END");
    flush_line();
    ExitProcess(0);
}
