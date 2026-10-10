/* C smoke test of the ABI: cc -Icore/include core/examples/smoke.c target/release/libsinhala_core.a -o smoke */
#include <stdio.h>
#include <string.h>
#include "sinhala_core.h"

int main(void) {
    struct SinhalaSession *s = sinhala_session_new();
    for (const char *p = "niladhaari"; *p; p++) sinhala_push_char(s, (uint32_t)*p);
    char *buf = sinhala_buffer(s);
    printf("buffer: %s\n", buf);
    sinhala_string_free(buf);
    uint32_t n = sinhala_row_count(s), sel = sinhala_selected_row(s);
    for (uint32_t i = 0; i < n; i++) {
        char *row = sinhala_row(s, i);
        printf("%s%u. %s\n", i == sel ? ">" : " ", i + 1, row);
        sinhala_string_free(row);
    }
    char *out = sinhala_commit(s);
    int ok = out && strlen(out) > 0;
    printf("commit: %s\n", out);
    sinhala_string_free(out);
    sinhala_session_free(s);
    return ok ? 0 : 1;
}
