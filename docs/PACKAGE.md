# PureLang packages

## Quick start

```bash
purec pkg init hello
cd hello   # if you created a subdir manually; init uses cwd
purec pkg build
./app
```

## Path dependencies

```bash
purec pkg add path:../shared_lib
```

Dependencies are recorded in `Pure.toml`. `pkg build` compiles the package entry point with `purec --compile`.

## Future

- Registry / versioned downloads
- Transitive dependency resolution
- `mod` path search across dependency roots
