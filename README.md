# Alinea

Alinea is a small macOS text editor for plain prose. Its first job is disciplined
hard wrapping: type or paste a paragraph, and Alinea keeps it reflowed to the
configured column, defaulting to 80.

The implementation is Rust plus `eframe`/`egui`. There is no JavaScript runtime
or web UI layer.

## Current Scope

- Plain UTF-8 text.
- One document per window.
- New, open, save, and save as.
- Literal search with next and previous.
- Automatic hard wrapping for prose paragraphs.
- Config stored at `~/Library/Application Support/Alinea/config.toml`.

## Development

```sh
script/dev.sh
script/test.sh
script/build_app.sh
script/install_local.sh
```

The local app bundle is written to `dist/Alinea.app`.

## License

GPL-3.0-or-later. See LICENSE.
