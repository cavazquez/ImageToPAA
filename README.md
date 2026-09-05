# ImageToPAA

Herramienta **open source** (Apache-2.0) para convertir imágenes PNG/TGA a
texturas `.paa` de Bohemia Interactive — alternativa al `ImageToPAA.exe` de
Arma 3 Tools.

Motivación: el pipeline de [RMTFAR](https://github.com/cavazquez/rmtfar) necesita
exports con **alfa suave** (siluetas de radio) en Linux/CI, sin depender de Wine
ni de Tools de Windows.

## Estado

Encoder DXT1/DXT5 funcional según el contrato en
[`docs/paa-profile.md`](docs/paa-profile.md):

- CLI `image-to-paa` + biblioteca `image_to_paa`
- PNG/TGA → RGBA8, dimensiones POT (sin resize implícito)
- Contenedor PAA (`01 FF` / `05 FF`) con tags `CGVA` / `CXAM` / `GALF` / `SFFO`
- Mipmaps alpha-correct / sRGB hasta eje menor 4
- Escritura atómica y `--force`

## Uso

```bash
cargo build --release
./target/release/image-to-paa artwork/radio.png dist/radio.paa
./target/release/image-to-paa icon.png icon.paa --format dxt5
./target/release/image-to-paa solid.png solid.paa --format auto --force
```

| Flag | Efecto |
|------|--------|
| `--format auto\|dxt1\|dxt5` | Formato canónico (`auto`: opaco→DXT1, alfa→DXT5) |
| `--dxt5` / `--dxt1` | Aliases; conflictivos entre sí y con `--format` (exit 2) |
| `--no-mips` | Sólo el nivel base |
| `--compress` | LZO1X por mip si reduce tamaño (compatible Arma) |
| `--force` | Sobrescribe una salida existente |

La salida debe terminar en `.paa`. Sin `--force`, un archivo existente no se
toca.

## RMTFAR (8 texturas)

```bash
./scripts/convert-rmtfar.sh /ruta/a/rmtfar /tmp/radio-paa
```

No modifica `addon/ui/radios`. Ver hashes en `SHA256SUMS` del directorio de
salida antes de promover.

## Interop oficial (opt-in)

```bash
export IMAGETOPAA_PATH=/path/to/ImageToPAA.exe
cargo build --release
./scripts/interop-imagetopaa.sh
```

No se descarga ni se exige en CI pública.

## Instalación

### Snap (Linux)

Cuando el nombre `imagetopaa` esté registrado en el Snap Store:

```bash
sudo snap install imagetopaa
imagetopaa input.png output.paa
```

Build local:

```bash
snapcraft
sudo snap install --dangerous imagetopaa_*.snap
```

### Release GitHub

1. Descargá el archivo de [Releases](https://github.com/cavazquez/ImageToPAA/releases) para tu OS.
2. Verificá checksum: `sha256sum -c image-to-paa-*.sha256`
3. Extraé y colocá `image-to-paa` en el `PATH`.

macOS aún no se publica (binarios sin firmar).

## Desarrollo

```bash
./check.sh          # fmt + clippy + tests + lockfile (igual que CI)
./check.sh --fix    # auto-format y luego el gate
```

Equivalente manual:

```bash
python3 scripts/generate-fixtures.py
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets --locked
```

MSRV: Rust **1.85** (Edition 2024).
