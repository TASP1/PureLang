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
