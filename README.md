# ImageToPAA

Herramienta **open source** (Apache-2.0) para convertir imágenes PNG/TGA a
texturas `.paa` de Bohemia Interactive — alternativa al `ImageToPAA.exe` de
Arma 3 Tools.

Motivación: el pipeline de [RMTFAR](https://github.com/cavazquez/rmtfar) necesita
exports con **alfa suave** (siluetas de radio) en Linux/CI, sin depender de Wine
ni de Tools de Windows.

## Estado

Scaffold inicial:

- CLI `image-to-paa`
- Lectura PNG/TGA (RGBA)
- Validación power-of-two
- Escritura **stub** (aún no es un PAA que TexView/Arma acepte)

Siguiente: compresión DXT1/DXT5 real + contenedor PAA compatible (ver
[`docs/paa-format.md`](docs/paa-format.md)).

## Uso (cuando el encoder esté listo)

```bash
cargo run --release -- artwork/radio.png dist/radio.paa
cargo run --release -- --dxt5 icon.png icon.paa
```

Flags:

| Flag | Efecto |
|------|--------|
| `--dxt5` | Fuerza DXT5 (alfa explícita) |
| `--dxt1` | Fuerza DXT1 |
| `--no-mips` | Sin cadena de mipmaps |

## Desarrollo

```bash
cargo test
cargo fmt
cargo clippy --all-targets -- -D warnings
```

## Relación con RMTFAR

Convive en `ProyectoRmtfar/ImageToPAA` junto a `rmtfar/`. Los scripts
`convert-radio-textures.*` de RMTFAR podrán apuntar a este binario en lugar de
`ImageToPAA.exe` cuando el encoder sea compatible.
