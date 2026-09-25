# PureLang UI

PureLang can build simple **web UIs** by emitting a self-contained HTML file.

## API

| Function | Args | Description |
|----------|------|-------------|
| `ui_begin(title, width, height)` | String, Number, Number | Start a window |
| `ui_label(text)` | String | Bold label row |
| `ui_text(text)` | String | Paragraph |
| `ui_button(label)` | String | Button (click → `alert`) |
| `ui_end()` | — | Write `purelang_ui.html` |

Also available as `std.ui_*` aliases where applicable.

## Example

```bash
purec --compile -o ui_demo examples/ui_demo.pure
./ui_demo
# open purelang_ui.html
```

## Roadmap

- Native window backends (raylib / OS toolkits)
- Event callbacks into PureLang
- Layout widgets (rows, columns, inputs)


## Native UI backends

`ui_native_available()` returns `0` until OS toolkits (Win32 / Cocoa / Android View) are wired.

Today the default backend remains **HTML export** (`ui_begin` … `ui_end` → `purelang_ui.html`).


## ui_window_show (v0.32+)

Win32: real HWND with static text and OK button. Other platforms: HTML export via ui_begin/end.
