/* PureLang runtime helpers — maps + minimal HTML UI */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <time.h>
#if defined(_WIN32)
#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#include <windows.h>
#include <wininet.h>
#ifndef strdup
#define strdup _strdup
#endif
#else
#include <unistd.h>
#include <errno.h>
#include <sys/types.h>
#include <sys/socket.h>
#include <netdb.h>
#include <pthread.h>
#endif

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

/* Native modal alert: Win32 MessageBox / macOS osascript / Linux zenity / HTML fallback */
void pl_ui_alert(char *title, char *msg) {
    const char *t = title ? title : "PureLang";
    const char *m = msg ? msg : "";
#if defined(_WIN32)
    MessageBoxA(NULL, m, t, MB_OK | MB_ICONINFORMATION);
#elif defined(__APPLE__)
    {
        char cmd[1024];
        snprintf(cmd, sizeof(cmd),
            "osascript -e 'display dialog \"%s\" with title \"%s\" buttons {\"OK\"} default button \"OK\"' 2>/dev/null",
            m, t);
        system(cmd);
    }
#else
    {
        char cmd[1024];
        snprintf(cmd, sizeof(cmd),
            "zenity --info --title='%s' --text='%s' 2>/dev/null || "
            "notify-send '%s' '%s' 2>/dev/null || true",
            t, m, t, m);
        system(cmd);
    }
#endif
    /* Always also append to HTML UI log if open */
    if (ui_fp && ui_open) {
        fprintf(ui_fp, "<div class=\"alert\"><strong>%s</strong>: %s</div>\n", t, m);
    }
}




#if defined(_WIN32)
static HWND pl_hwnd = NULL;
static HWND pl_last_hwnd = NULL;
static char pl_ui_title[256] = "PureLang";
static char pl_ui_body[2048] = "";

static LRESULT CALLBACK pl_wnd_proc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam) {
    switch (msg) {
    case WM_COMMAND:
        if (LOWORD(wParam) == 1) {
            MessageBoxA(hwnd, "Button clicked", pl_ui_title, MB_OK);
        }
        break;
    case WM_CLOSE:
        DestroyWindow(hwnd);
        break;
    case WM_DESTROY:
        PostQuitMessage(0);
        break;
    default:
        return DefWindowProcA(hwnd, msg, wParam, lParam);
    }
    return 0;
}

/* Show a real Win32 window with label + button (message loop until closed). */
int64_t pl_ui_window_show(char *title, char *body) {
    if (title) {
        strncpy(pl_ui_title, title, sizeof(pl_ui_title) - 1);
        pl_ui_title[sizeof(pl_ui_title) - 1] = '\0';
    }
    if (body) {
        strncpy(pl_ui_body, body, sizeof(pl_ui_body) - 1);
        pl_ui_body[sizeof(pl_ui_body) - 1] = '\0';
    }
    HINSTANCE hi = GetModuleHandleA(NULL);
    WNDCLASSA wc;
    memset(&wc, 0, sizeof(wc));
    wc.lpfnWndProc = pl_wnd_proc;
    wc.hInstance = hi;
    wc.lpszClassName = "PureLangWin";
    wc.hbrBackground = (HBRUSH)(COLOR_WINDOW + 1);
    wc.hCursor = LoadCursor(NULL, IDC_ARROW);
    RegisterClassA(&wc);
    HWND hwnd = CreateWindowExA(
        0, "PureLangWin", pl_ui_title,
        WS_OVERLAPPEDWINDOW | WS_VISIBLE,
        CW_USEDEFAULT, CW_USEDEFAULT, 420, 240,
        NULL, NULL, hi, NULL);
    if (!hwnd) return 0;
    pl_last_hwnd = hwnd;
    CreateWindowExA(0, "STATIC", pl_ui_body,
        WS_CHILD | WS_VISIBLE, 12, 12, 380, 120,
        hwnd, NULL, hi, NULL);
    CreateWindowExA(0, "BUTTON", "OK",
        WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON, 160, 150, 80, 28,
        hwnd, (HMENU)1, hi, NULL);
    ShowWindow(hwnd, SW_SHOW);
    UpdateWindow(hwnd);
    MSG msg;
    while (GetMessageA(&msg, NULL, 0, 0) > 0) {
        TranslateMessage(&msg);
        DispatchMessageA(&msg);
    }
    return 1;
}
#else
/* Non-Windows: open HTML UI file or print body */
int64_t pl_ui_window_show(char *title, char *body) {
    pl_ui_begin(title ? title : "PureLang", 420, 240);
    pl_ui_label(body ? body : "");
    pl_ui_button("OK");
    pl_ui_end();
    return 1;
}
#endif



#if defined(_WIN32)
/* Additional Win32 widgets attached to last shown parent or create child controls API */
static HWND pl_last_hwnd = NULL;

void pl_ui_add_button(char *label) {
    if (!pl_last_hwnd || !label) return;
    CreateWindowExA(0, "BUTTON", label, WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
        20, 180, 100, 28, pl_last_hwnd, (HMENU)2, GetModuleHandleA(NULL), NULL);
}
void pl_ui_add_edit(char *placeholder) {
    if (!pl_last_hwnd) return;
    CreateWindowExA(WS_EX_CLIENTEDGE, "EDIT", placeholder ? placeholder : "",
        WS_CHILD | WS_VISIBLE | ES_LEFT | ES_AUTOHSCROLL,
        12, 140, 380, 24, pl_last_hwnd, (HMENU)3, GetModuleHandleA(NULL), NULL);
}
void pl_ui_add_listbox(void) {
    if (!pl_last_hwnd) return;
    CreateWindowExA(WS_EX_CLIENTEDGE, "LISTBOX", "",
        WS_CHILD | WS_VISIBLE | LBS_NOTIFY | WS_VSCROLL,
        12, 40, 180, 90, pl_last_hwnd, (HMENU)4, GetModuleHandleA(NULL), NULL);
}
#else
void pl_ui_add_button(char *label) { pl_ui_button(label); }
void pl_ui_add_edit(char *placeholder) {
    if (ui_fp && ui_open)
        fprintf(ui_fp, "<input type=\"text\" placeholder=\"%s\"/>\n", placeholder ? placeholder : "");
}
void pl_ui_add_listbox(void) {
    if (ui_fp && ui_open)
        fprintf(ui_fp, "<select><option>Item</option></select>\n");
}
#endif

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


int64_t pl_str_char_at(char *s, int64_t i) {
    if (!s || i < 0) return -1;
    size_t n = strlen(s);
    if ((size_t)i >= n) return -1;
    return (unsigned char)s[i];
}

char *pl_str_slice(char *s, int64_t start, int64_t end) {
    if (!s) return (char *)calloc(1, 1);
    size_t n = strlen(s);
    if (start < 0) start = 0;
    if (end < start) end = start;
    if ((size_t)start > n) start = (int64_t)n;
    if ((size_t)end > n) end = (int64_t)n;
    size_t len = (size_t)(end - start);
    char *out = (char *)malloc(len + 1);
    if (!out) return (char *)calloc(1, 1);
    memcpy(out, s + start, len);
    out[len] = '\0';
    return out;
}

int64_t pl_time_ms(void) {
#if defined(_WIN32)
    return (int64_t)GetTickCount64();
#else
    struct timespec ts;
    if (clock_gettime(CLOCK_MONOTONIC, &ts) != 0) return 0;
    return (int64_t)ts.tv_sec * 1000 + (int64_t)ts.tv_nsec / 1000000;
#endif
}

#if defined(_WIN32)
void pl_sleep_ms(int64_t ms) { if (ms > 0) Sleep((DWORD)ms); }
/* In-process HTTPS/HTTP via WinInet (schannel under the hood) */
static char *pl_http_get_wininet(char *url) {
    char *empty = (char *)calloc(1, 1);
    if (!url) return empty;
    HINTERNET hNet = InternetOpenA("PureLang/0.31", INTERNET_OPEN_TYPE_PRECONFIG, NULL, NULL, 0);
    if (!hNet) return empty;
    HINTERNET hUrl = InternetOpenUrlA(hNet, url, NULL, 0,
        INTERNET_FLAG_RELOAD | INTERNET_FLAG_NO_CACHE_WRITE | INTERNET_FLAG_SECURE, 0);
    if (!hUrl) {
        /* retry without SECURE flag for plain http */
        hUrl = InternetOpenUrlA(hNet, url, NULL, 0,
            INTERNET_FLAG_RELOAD | INTERNET_FLAG_NO_CACHE_WRITE, 0);
    }
    if (!hUrl) { InternetCloseHandle(hNet); return empty; }
    size_t cap = 8192, len = 0;
    char *buf = (char *)malloc(cap);
    if (!buf) { InternetCloseHandle(hUrl); InternetCloseHandle(hNet); return empty; }
    for (;;) {
        DWORD n = 0;
        if (len + 4096 + 1 > cap) {
            cap *= 2;
            char *nb = (char *)realloc(buf, cap);
            if (!nb) break;
            buf = nb;
        }
        if (!InternetReadFile(hUrl, buf + len, 4096, &n) || n == 0) break;
        len += (size_t)n;
    }
    buf[len] = '\0';
    InternetCloseHandle(hUrl);
    InternetCloseHandle(hNet);
    free(empty);
    return buf;
}
char *pl_http_get(char *url) { return pl_http_get_wininet(url); }

#define PL_CHAN_CAP 64
typedef struct {
    CRITICAL_SECTION mu;
    CONDITION_VARIABLE cv_not_full;
    CONDITION_VARIABLE cv_not_empty;
    int64_t q[PL_CHAN_CAP];
    int head, tail, count;
} PLChanWin;

typedef struct {
    PLChanWin *ch;
    int64_t delay_ms;
    int64_t value;
} PLThreadSendArgsWin;

typedef int64_t (*PLFn0)(void);

void *pl_chan_new(void) {
    PLChanWin *c = (PLChanWin *)calloc(1, sizeof(PLChanWin));
    if (!c) return NULL;
    InitializeCriticalSection(&c->mu);
    InitializeConditionVariable(&c->cv_not_full);
    InitializeConditionVariable(&c->cv_not_empty);
    return c;
}

void pl_chan_send(void *ch, int64_t v) {
    PLChanWin *c = (PLChanWin *)ch;
    if (!c) return;
    EnterCriticalSection(&c->mu);
    while (c->count == PL_CHAN_CAP)
        SleepConditionVariableCS(&c->cv_not_full, &c->mu, INFINITE);
    c->q[c->tail] = v;
    c->tail = (c->tail + 1) % PL_CHAN_CAP;
    c->count++;
    WakeConditionVariable(&c->cv_not_empty);
    LeaveCriticalSection(&c->mu);
}

int64_t pl_chan_recv(void *ch) {
    PLChanWin *c = (PLChanWin *)ch;
    if (!c) return 0;
    EnterCriticalSection(&c->mu);
    while (c->count == 0)
        SleepConditionVariableCS(&c->cv_not_empty, &c->mu, INFINITE);
    int64_t v = c->q[c->head];
    c->head = (c->head + 1) % PL_CHAN_CAP;
    c->count--;
    WakeConditionVariable(&c->cv_not_full);
    LeaveCriticalSection(&c->mu);
    return v;
}

int64_t pl_chan_len(void *ch) {
    PLChanWin *c = (PLChanWin *)ch;
    if (!c) return 0;
    EnterCriticalSection(&c->mu);
    int64_t n = c->count;
    LeaveCriticalSection(&c->mu);
    return n;
}

static DWORD WINAPI pl_thread_send_main_win(LPVOID arg) {
    PLThreadSendArgsWin *a = (PLThreadSendArgsWin *)arg;
    if (a->delay_ms > 0) pl_sleep_ms(a->delay_ms);
    pl_chan_send(a->ch, a->value);
    free(a);
    return 0;
}

void pl_thread_spawn_send(void *ch, int64_t delay_ms, int64_t value) {
    PLThreadSendArgsWin *a = (PLThreadSendArgsWin *)malloc(sizeof(PLThreadSendArgsWin));
    if (!a) return;
    a->ch = (PLChanWin *)ch;
    a->delay_ms = delay_ms;
    a->value = value;
    HANDLE h = CreateThread(NULL, 0, pl_thread_send_main_win, a, 0, NULL);
    if (h)
        CloseHandle(h);
    else {
        pl_chan_send(a->ch, a->value);
        free(a);
    }
}

static DWORD WINAPI pl_thread_fn_main_win(LPVOID arg) {
    PLFn0 f = (PLFn0)arg;
    if (f) f();
    return 0;
}

void pl_thread_spawn(void *fn) {
    if (!fn) return;
    HANDLE h = CreateThread(NULL, 0, pl_thread_fn_main_win, fn, 0, NULL);
    if (h)
        CloseHandle(h);
    else
        ((PLFn0)fn)();
}

int64_t pl_ui_native_available(void) { return 1; }
#else

void pl_sleep_ms(int64_t ms) {
    if (ms <= 0) return;
    struct timespec ts;
    ts.tv_sec = (time_t)(ms / 1000);
    ts.tv_nsec = (long)((ms % 1000) * 1000000L);
    while (nanosleep(&ts, &ts) != 0 && errno == EINTR) {}
}

#if defined(PURELANG_HAVE_CURL) || (defined(__has_include) && __has_include(<curl/curl.h>))
#include <curl/curl.h>
static size_t pl_curl_write(void *ptr, size_t size, size_t nmemb, void *userdata) {
    size_t n = size * nmemb;
    char **pbuf = (char **)userdata;
    size_t old = *pbuf ? strlen(*pbuf) : 0;
    char *nb = (char *)realloc(*pbuf, old + n + 1);
    if (!nb) return 0;
    memcpy(nb + old, ptr, n);
    nb[old + n] = '\0';
    *pbuf = nb;
    return n;
}
static char *pl_http_get_libcurl(const char *url) {
    CURL *c = curl_easy_init();
    if (!c) return (char *)calloc(1, 1);
    char *buf = (char *)calloc(1, 1);
    curl_easy_setopt(c, CURLOPT_URL, url);
    curl_easy_setopt(c, CURLOPT_FOLLOWLOCATION, 1L);
    curl_easy_setopt(c, CURLOPT_TIMEOUT, 30L);
    curl_easy_setopt(c, CURLOPT_WRITEFUNCTION, pl_curl_write);
    curl_easy_setopt(c, CURLOPT_WRITEDATA, &buf);
    curl_easy_setopt(c, CURLOPT_USERAGENT, "PureLang/0.31");
    if (curl_easy_perform(c) != CURLE_OK) {
        free(buf);
        buf = (char *)calloc(1, 1);
    }
    curl_easy_cleanup(c);
    return buf;
}
#endif


#if !defined(_WIN32)
#include <openssl/ssl.h>
#include <openssl/err.h>

static char *pl_http_get_openssl(const char *url) {
    char *empty = (char *)calloc(1, 1);
    if (!url || strncmp(url, "https://", 8) != 0) return empty;
    const char *p = url + 8;
    char host[256], path[1024];
    int port = 443;
    const char *slash = strchr(p, '/');
    const char *colon = strchr(p, ':');
    size_t host_len;
    if (colon && (!slash || colon < slash)) {
        host_len = (size_t)(colon - p);
        if (host_len >= sizeof(host)) host_len = sizeof(host) - 1;
        memcpy(host, p, host_len); host[host_len] = '\0';
        port = atoi(colon + 1);
        if (slash) { strncpy(path, slash, sizeof(path)-1); path[sizeof(path)-1]='\0'; }
        else strcpy(path, "/");
    } else if (slash) {
        host_len = (size_t)(slash - p);
        if (host_len >= sizeof(host)) host_len = sizeof(host) - 1;
        memcpy(host, p, host_len); host[host_len] = '\0';
        strncpy(path, slash, sizeof(path)-1); path[sizeof(path)-1]='\0';
    } else {
        strncpy(host, p, sizeof(host)-1); host[sizeof(host)-1]='\0';
        strcpy(path, "/");
    }
    struct addrinfo hints, *res = NULL;
    memset(&hints, 0, sizeof(hints));
    hints.ai_family = AF_UNSPEC;
    hints.ai_socktype = SOCK_STREAM;
    char port_s[16];
    snprintf(port_s, sizeof(port_s), "%d", port);
    if (getaddrinfo(host, port_s, &hints, &res) != 0 || !res) return empty;
    int fd = -1;
    for (struct addrinfo *ai = res; ai; ai = ai->ai_next) {
        fd = (int)socket(ai->ai_family, ai->ai_socktype, ai->ai_protocol);
        if (fd < 0) continue;
        if (connect(fd, ai->ai_addr, ai->ai_addrlen) == 0) break;
        close(fd); fd = -1;
    }
    freeaddrinfo(res);
    if (fd < 0) return empty;

    SSL_library_init();
    SSL_load_error_strings();
    const SSL_METHOD *method = TLS_client_method();
    SSL_CTX *ctx = SSL_CTX_new(method);
    if (!ctx) { close(fd); return empty; }
    SSL_CTX_set_default_verify_paths(ctx);
    /* VERIFY_PEER once CA bundle is guaranteed; NONE keeps demos usable */
    SSL_CTX_set_verify(ctx, SSL_VERIFY_NONE, NULL);
    SSL *ssl = SSL_new(ctx);
    SSL_set_fd(ssl, fd);
    SSL_set_tlsext_host_name(ssl, host);
    if (SSL_connect(ssl) != 1) {
        SSL_free(ssl); SSL_CTX_free(ctx); close(fd); return empty;
    }
    char req[2048];
    snprintf(req, sizeof(req),
        "GET %s HTTP/1.0\r\nHost: %s\r\nUser-Agent: PureLang/0.33\r\nConnection: close\r\n\r\n",
        path, host);
    SSL_write(ssl, req, (int)strlen(req));
    size_t cap = 8192, len = 0;
    char *buf = (char *)malloc(cap);
    if (!buf) { SSL_shutdown(ssl); SSL_free(ssl); SSL_CTX_free(ctx); close(fd); return empty; }
    for (;;) {
        if (len + 4096 + 1 > cap) {
            cap *= 2;
            char *nb = (char *)realloc(buf, cap);
            if (!nb) break;
            buf = nb;
        }
        int n = SSL_read(ssl, buf + len, 4096);
        if (n <= 0) break;
        len += (size_t)n;
    }
    buf[len] = '\0';
    SSL_shutdown(ssl);
    SSL_free(ssl);
    SSL_CTX_free(ctx);
    close(fd);
    char *body = strstr(buf, "\r\n\r\n");
    if (body) {
        body += 4;
        size_t blen = strlen(body);
        char *out = (char *)malloc(blen + 1);
        if (out) { memcpy(out, body, blen + 1); free(buf); free(empty); return out; }
    }
    free(empty);
    return buf;
}
#endif

static char *pl_http_get_curl(const char *url) {
#if defined(PURELANG_HAVE_CURL) || (defined(__has_include) && __has_include(<curl/curl.h>))
    return pl_http_get_libcurl(url);
#else

    char cmd[2048];
    snprintf(cmd, sizeof(cmd), "curl -fsSL --max-time 30 '%s' 2>/dev/null", url);
    FILE *fp = popen(cmd, "r");
    if (!fp) return (char *)calloc(1, 1);
    size_t cap = 8192, len = 0;
    char *buf = (char *)malloc(cap);
    if (!buf) { pclose(fp); return (char *)calloc(1, 1); }
    for (;;) {
        if (len + 2048 > cap) {
            cap *= 2;
            char *nbuf = (char *)realloc(buf, cap);
            if (!nbuf) break;
            buf = nbuf;
        }
        size_t n = fread(buf + len, 1, 2047, fp);
        if (n == 0) break;
        len += n;
    }
    pclose(fp);
    buf[len] = '\0';
    return buf;
#endif
}

char *pl_http_get(char *url) {
    char *empty = (char *)calloc(1, 1);
    if (!url) return empty;
    if (strncmp(url, "https://", 8) == 0) {
        free(empty);
#if !defined(_WIN32)
        return pl_http_get_openssl(url);
#else
        return pl_http_get_curl(url);
#endif
    }
    if (strncmp(url, "http://", 7) != 0) return empty;

    char host[256], path[1024];
    int port = 80;
    const char *p = url + 7;
    const char *slash = strchr(p, '/');
    const char *colon = strchr(p, ':');
    size_t host_len;
    if (colon && (!slash || colon < slash)) {
        host_len = (size_t)(colon - p);
        if (host_len >= sizeof(host)) host_len = sizeof(host) - 1;
        memcpy(host, p, host_len); host[host_len] = '\0';
        port = atoi(colon + 1);
        if (slash) { strncpy(path, slash, sizeof(path)-1); path[sizeof(path)-1]='\0'; }
        else strcpy(path, "/");
    } else if (slash) {
        host_len = (size_t)(slash - p);
        if (host_len >= sizeof(host)) host_len = sizeof(host) - 1;
        memcpy(host, p, host_len); host[host_len] = '\0';
        strncpy(path, slash, sizeof(path)-1); path[sizeof(path)-1]='\0';
    } else {
        strncpy(host, p, sizeof(host)-1); host[sizeof(host)-1]='\0';
        strcpy(path, "/");
    }

    struct addrinfo hints, *res = NULL;
    memset(&hints, 0, sizeof(hints));
    hints.ai_family = AF_UNSPEC;
    hints.ai_socktype = SOCK_STREAM;
    char port_s[16];
    snprintf(port_s, sizeof(port_s), "%d", port);
    if (getaddrinfo(host, port_s, &hints, &res) != 0 || !res) return empty;

    int fd = -1;
    for (struct addrinfo *ai = res; ai; ai = ai->ai_next) {
        fd = (int)socket(ai->ai_family, ai->ai_socktype, ai->ai_protocol);
        if (fd < 0) continue;
        if (connect(fd, ai->ai_addr, ai->ai_addrlen) == 0) break;
        close(fd); fd = -1;
    }
    freeaddrinfo(res);
    if (fd < 0) return empty;

    char req[2048];
    snprintf(req, sizeof(req),
        "GET %s HTTP/1.0\r\nHost: %s\r\nConnection: close\r\n\r\n", path, host);
    if (send(fd, req, strlen(req), 0) < 0) { close(fd); return empty; }

    size_t cap = 8192, len = 0;
    char *buf = (char *)malloc(cap);
    if (!buf) { close(fd); return empty; }
    for (;;) {
        if (len + 2048 > cap) {
            cap *= 2;
            char *nbuf = (char *)realloc(buf, cap);
            if (!nbuf) break;
            buf = nbuf;
        }
        ssize_t n = recv(fd, buf + len, 2047, 0);
        if (n <= 0) break;
        len += (size_t)n;
    }
    close(fd);
    buf[len] = '\0';
    char *body = strstr(buf, "\r\n\r\n");
    if (body) {
        body += 4;
        size_t blen = strlen(body);
        char *out = (char *)malloc(blen + 1);
        if (!out) { free(buf); return empty; }
        memcpy(out, body, blen + 1);
        free(buf); free(empty);
        return out;
    }
    free(empty);
    return buf;
}

#define PL_CHAN_CAP 64
typedef struct {
    pthread_mutex_t mu;
    pthread_cond_t cv;
    int64_t q[PL_CHAN_CAP];
    int head, tail, count;
} PLChan;

typedef struct {
    PLChan *ch;
    int64_t delay_ms;
    int64_t value;
} PLThreadSendArgs;

void *pl_chan_new(void) {
    PLChan *c = (PLChan *)calloc(1, sizeof(PLChan));
    if (!c) return NULL;
    pthread_mutex_init(&c->mu, NULL);
    pthread_cond_init(&c->cv, NULL);
    return c;
}

void pl_chan_send(void *ch, int64_t v) {
    PLChan *c = (PLChan *)ch;
    if (!c) return;
    pthread_mutex_lock(&c->mu);
    while (c->count == PL_CHAN_CAP)
        pthread_cond_wait(&c->cv, &c->mu);
    c->q[c->tail] = v;
    c->tail = (c->tail + 1) % PL_CHAN_CAP;
    c->count++;
    pthread_cond_signal(&c->cv);
    pthread_mutex_unlock(&c->mu);
}

int64_t pl_chan_recv(void *ch) {
    PLChan *c = (PLChan *)ch;
    if (!c) return 0;
    pthread_mutex_lock(&c->mu);
    while (c->count == 0)
        pthread_cond_wait(&c->cv, &c->mu);
    int64_t v = c->q[c->head];
    c->head = (c->head + 1) % PL_CHAN_CAP;
    c->count--;
    pthread_cond_signal(&c->cv);
    pthread_mutex_unlock(&c->mu);
    return v;
}

int64_t pl_chan_len(void *ch) {
    PLChan *c = (PLChan *)ch;
    if (!c) return 0;
    pthread_mutex_lock(&c->mu);
    int64_t n = c->count;
    pthread_mutex_unlock(&c->mu);
    return n;
}

static void *pl_thread_send_main(void *arg) {
    PLThreadSendArgs *a = (PLThreadSendArgs *)arg;
    if (a->delay_ms > 0) pl_sleep_ms(a->delay_ms);
    pl_chan_send(a->ch, a->value);
    free(a);
    return NULL;
}

void pl_thread_spawn_send(void *ch, int64_t delay_ms, int64_t value) {
    PLThreadSendArgs *a = (PLThreadSendArgs *)malloc(sizeof(PLThreadSendArgs));
    if (!a) return;
    a->ch = (PLChan *)ch;
    a->delay_ms = delay_ms;
    a->value = value;
    pthread_t t;
    if (pthread_create(&t, NULL, pl_thread_send_main, a) == 0)
        pthread_detach(t);
    else
        free(a);
}

typedef int64_t (*PLFn0)(void);

static void *pl_thread_fn_main(void *arg) {
    PLFn0 f = (PLFn0)arg;
    if (f) f();
    return NULL;
}

void pl_thread_spawn(void *fn) {
    if (!fn) return;
    pthread_t t;
    if (pthread_create(&t, NULL, pl_thread_fn_main, fn) == 0)
        pthread_detach(t);
    else
        ((PLFn0)fn)(); /* fallback: run on caller thread */
}

int64_t pl_ui_native_available(void) { return 1; }
#endif


int64_t pl_ui_android_available(void) {
#if defined(__ANDROID__)
    return 1;
#else
    return 0;
#endif
}
int64_t pl_ui_cocoa_available(void) {
#if defined(__APPLE__)
    return 1;
#else
    return 0;
#endif
}

