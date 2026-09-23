# PureLang VS Code (generic LSP)

There is no marketplace extension yet. Use any **Generic LSP** client:

```json
{
  "language": "purelang",
  "command": ["purec", "--lsp"],
  "filetypes": ["pure"]
}
```

Or with the "LSP Client" / "LanguageClient" style extensions, set the server command to `purec --lsp`.

Associate `*.pure` with a language id of your choice and enable the client for that id.
