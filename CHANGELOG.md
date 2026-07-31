# Changelog

## Unreleased

- Pin `image` a `<0.25.10` para respetar MSRV 1.85 (`image` 0.25.10 exige rustc 1.88)
- CI: el job `lockfile` ya no redirige a `/dev/null` (rompe en PowerShell/Windows)

## 0.1.0

- Encoder DXT1/DXT5 real (perfil MVP en `docs/paa-profile.md`)
- CLI: `--format`, `--force`, `--compress` (LZO1X opcional), escritura atómica
- Biblioteca `image_to_paa` reutilizable
- Fixtures sintéticos + CI (fmt/clippy/test/MSRV 1.85) + release por tag
- Script `scripts/convert-rmtfar.sh` para las 8 texturas RMTFAR
- Snapcraft (`imagetopaa`) + `./check.sh`
- Edition **2024**
