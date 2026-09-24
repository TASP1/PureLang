/* PureLang runtime helpers — maps + minimal HTML UI */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <time.h>

#define PL_MAP_CAP 128

typedef struct {
    char *keys[PL_MAP_CAP];
    int64_t vals[PL_MAP_CAP];
    int n;
} PLMap;

PLMap *pl_map_new(void) {
    PLMap *m = (PLMap *)calloc(1, sizeof(PLMap));
    return m;
}

void pl_map_set(PLMap *m, char *key, int64_t val) {
    if (!m || !key) return;
    for (int i = 0; i < m->n; i++) {
        if (m->keys[i] && strcmp(m->keys[i], key) == 0) {
            m->vals[i] = val;
            return;
        }
    }
    if (m->n >= PL_MAP_CAP) return;
    m->keys[m->n] = strdup(key);
    m->vals[m->n] = val;
    m->n++;
}

int64_t pl_map_get(PLMap *m, char *key) {
    if (!m || !key) return 0;
    for (int i = 0; i < m->n; i++) {
        if (m->keys[i] && strcmp(m->keys[i], key) == 0)
            return m->vals[i];
    }
    return 0;
}

int64_t pl_map_has(PLMap *m, char *key) {
    if (!m || !key) return 0;
    for (int i = 0; i < m->n; i++) {
        if (m->keys[i] && strcmp(m->keys[i], key) == 0)
            return 1;
    }
    return 0;
}

int64_t pl_map_len(PLMap *m) {
    return m ? (int64_t)m->n : 0;
}

/* ---- Minimal UI: accumulates HTML, writes purelang_ui.html on end ---- */
static FILE *ui_fp = NULL;
static int ui_open = 0;

void pl_ui_begin(char *title, int64_t w, int64_t h) {
    if (ui_fp) fclose(ui_fp);
    ui_fp = fopen("purelang_ui.html", "w");
    if (!ui_fp) return;
    ui_open = 1;
    fprintf(ui_fp,
        "<!DOCTYPE html>\n<html><head><meta charset=\"utf-8\">\n"
        "<title>%s</title>\n"
        "<style>\n"
        "body{font-family:system-ui,sans-serif;margin:0;background:#0f172a;color:#e2e8f0;"
        "display:flex;justify-content:center;padding:2rem;}\n"
        ".win{width:%lldpx;min-height:%lldpx;background:#1e293b;border-radius:12px;"
        "box-shadow:0 20px 50px rgba(0,0,0,.4);padding:1.5rem;}\n"
        "h1{margin:0 0 1rem;font-size:1.4rem;color:#38bdf8;}\n"
        "p{margin:.5rem 0;line-height:1.5;}\n"
        "button{background:#38bdf8;color:#0f172a;border:none;border-radius:8px;"
        "padding:.6rem 1.2rem;font-weight:600;margin:.4rem .4rem .4rem 0;cursor:pointer;}\n"
        "button:hover{background:#7dd3fc;}\n"
        ".row{margin:.75rem 0;}\n"
        "</style>\n</head><body>\n<div class=\"win\">\n"
        "<h1>%s</h1>\n",
        title ? title : "PureLang UI",
        (long long)(w > 0 ? w : 480),
        (long long)(h > 0 ? h : 320),
        title ? title : "PureLang UI");
}

void pl_ui_text(char *s) {
    if (!ui_fp || !ui_open) return;
    fprintf(ui_fp, "<p>%s</p>\n", s ? s : "");
}

void pl_ui_button(char *label) {
    if (!ui_fp || !ui_open) return;
    fprintf(ui_fp,
        "<button onclick=\"alert('%s')\">%s</button>\n",
        label ? label : "Button",
        label ? label : "Button");
}

void pl_ui_label(char *s) {
    if (!ui_fp || !ui_open) return;
    fprintf(ui_fp, "<div class=\"row\"><strong>%s</strong></div>\n", s ? s : "");
}

int64_t pl_ui_end(void) {
    if (!ui_fp || !ui_open) return 0;
    fprintf(ui_fp,
        "</div>\n<script>console.log('PureLang UI ready');</script>\n"
        "</body></html>\n");
    fclose(ui_fp);
    ui_fp = NULL;
    ui_open = 0;
    return 1;
}


/* ---- String helpers ---- */
int64_t pl_str_contains(char *hay, char *needle) {
    if (!hay || !needle) return 0;
    return strstr(hay, needle) != NULL ? 1 : 0;
}

int64_t pl_str_eq(char *a, char *b) {
    if (!a && !b) return 1;
    if (!a || !b) return 0;
    return strcmp(a, b) == 0 ? 1 : 0;
}

char *pl_str_concat(char *a, char *b) {
    size_t la = a ? strlen(a) : 0;
    size_t lb = b ? strlen(b) : 0;
    char *out = (char *)malloc(la + lb + 1);
    if (!out) return NULL;
    if (a) memcpy(out, a, la); else la = 0;
    if (b) memcpy(out + la, b, lb);
    out[la + lb] = '\0';
    return out;
}

char *pl_str_from_num(int64_t n) {
    char *out = (char *)malloc(32);
    if (!out) return NULL;
    snprintf(out, 32, "%lld", (long long)n);
    return out;
}


int64_t pl_time_ms(void) {
    struct timespec ts;
    if (clock_gettime(CLOCK_MONOTONIC, &ts) != 0) {
        return 0;
    }
    return (int64_t)ts.tv_sec * 1000 + (int64_t)ts.tv_nsec / 1000000;
}
