# Perfil PAA MVP (DXT) — contrato normativo

Este documento fija el comportamiento observable del encoder `image-to-paa`
(biblioteca + CLI) para texturas DXT1/DXT5 de Arma 3. Sustituye el roadmap
informal de `docs/paa-format.md` como fuente de verdad del MVP.

**Licencia:** el proyecto es Apache-2.0. No se copiará código GPL de
[`armake`](https://github.com/KoffeinFlummi/armake) ni de otros derivados GPL.
La implementación se basa en documentación pública, dumps hexadecimales
propios y referencias MIT (p. ej. la descripción de layout en
[`woozymasta/paa`](https://github.com/woozymasta/paa)).

## Fuentes y evidencia

| Afirmación | Fuente / evidencia |
|---|---|
| Firma DXT1 = bytes `01 FF` (LE `0xFF01`) | [BI Wiki — PAA File Format](https://community.bohemia.net/wiki/PAA_File_Format); dumps comunitarios; `woozymasta/paa` docs |
| Firma DXT5 = bytes `05 FF` (LE `0xFF05`) | Idem |
| Tags `GGAT` + nombre 4 + `u32` LE + payload | BI Wiki; layout documentado en `woozymasta/paa` |
| Nombres canónicos `CGVA`, `CXAM`, `GALF`, `SFFO` (orden de escritura BI) | Hex de herramientas BI / ImageToPAA; `woozymasta/paa` README+docs |
| `CGVA`/`CXAM` payload BGRA (no RGBA) | Observado en salidas BI; documentado en `woozymasta/paa` |
| `CXAM` = `FF FF FF FF` en texturas DXT de herramientas BI | Observado en oráculos ImageToPAA; documentado como política BI |
| `GALF` = `01 00 00 00` en DXT5 con alfa interpolada | Hex ImageToPAA / TexView; valor `1` = interpolated alpha |
| `SFFO` = 16 × `u32` LE offsets absolutos al inicio de cada mip | BI Wiki; tamaño de tag 64 bytes |
| Paleta vacía = `00 00` tras los tags (DXT) | Layout DXT documentado; hex de archivos DXT sin índices |
| Bloque mip = `u16` w + `u16` h + `u24` LE len + payload | BI Wiki |
| Bit alto de width (`0x8000`) = mip comprimido LZO | BI Wiki; Arma 2+ |
| Cadena DXT hasta eje menor = 4 px | Comportamiento ImageToPAA / TexConvert; cadena 1024×2048 → … → 4×8 |
| BC1/BC3 = S3TC | [Khronos S3TC](https://wikis.khronos.org/opengl/S3_Texture_Compression) |

Donde la evidencia es “observado en herramientas BI” y aún no hay un dump
versionado en este repo, el comportamiento se trata como **requisito MVP
provisional** y se confirmará con el oráculo de `#2` / suite `#12`.

## Alcance del MVP

**Incluido**

- Contenedor DXT1 (`01 FF`) y DXT5 (`05 FF`).
- Tags `CGVA`, `CXAM`, `GALF` (condicional), `SFFO`.
- Paleta vacía.
- Cadena de mipmaps DXT hasta eje menor 4.
- Payloads BC1 (8 B/bloque) y BC3 (16 B/bloque) sin LZO.
- CLI: PNG/TGA → `.paa`, formatos `auto|dxt1|dxt5`, `--no-mips`, `--force`.

**Fuera del MVP (extensiones futuras)**

- DXT2/3/4, ARGB8888/1555/4444, AI88/GRAYA.
- LZO por mip (`#13`).
- Normal maps / tag `ZIWS` / swizzle `_nohq`.
- Reglas completas de `TexConvert.cfg` / hints por sufijo.
- Tags `PROC` u otros no listados.

**No confirmado aún (documentado, no bloqueante si el oráculo difiere)**

- Padding final exacto tras el último mip (herramientas BI escriben 6 bytes
  cero; algunos lectores también aceptan un mip dummy `0×0`). El MVP escribe
  **6 bytes `00`** tras el último mip, alineado con el layout documentado de
  herramientas modernas compatibles.
- Valor `GALF=2` para alfa “binaria” en DXT5: **no se emite** en el MVP.
  DXT5 siempre usa `GALF=1` cuando el tag está presente. DXT1 **omite** `GALF`.

## Layout binario (byte a byte)

```
offset 0:
  u16 LE type                 ; 0xFF01 → bytes 01 FF | 0xFF05 → bytes 05 FF

tags (0..N), cada uno:
  "GGAT"                      ; 4 bytes ASCII
  name[4]                     ; p.ej. "CGVA"
  u32 LE payload_len
  payload[payload_len]

palette:
  u16 LE = 0                  ; bytes 00 00  (DXT: sin colores de paleta)

mips (1..16):
  u16 LE width                ; bit 15 = LZO (MVP: siempre 0)
  u16 LE height
  u24 LE data_len             ; 3 bytes little-endian, max 0xFFFFFF
  payload[data_len]           ; bloques BC1 o BC3

trailer:
  00 00 00 00 00 00           ; 6 bytes cero (padding / marca de fin práctica)
```

### Orden canónico de tags (MVP)

1. `CGVA` — promedio BGRA de la imagen **base** (4 bytes).
2. `CXAM` — máximo BGRA; para DXT del MVP: siempre `FF FF FF FF`.
3. `GALF` — sólo si formato = DXT5: `01 00 00 00`.
4. `SFFO` — 64 bytes = 16 offsets `u32` LE absolutos desde el inicio del
   archivo hasta el primer byte del `width` de cada mip. Entradas sin mip = `0`.

Tamaño de un tag con payload de 4 bytes: `4+4+4+4 = 16`.
Tamaño de `SFFO`: `4+4+4+64 = 76`.

### Cálculo de offsets `SFFO`

```
off0 = 2
     + size(CGVA) + size(CXAM) + size(GALF?) + size(SFFO)
     + 2                          ; paleta
# cada size(tag) = 12 + len(payload)

para i en 0..mip_count-1:
  SFFO[i] = off_i
  off_{i+1} = off_i + 2 + 2 + 3 + len(payload_i)
SFFO[mip_count..15] = 0
```

### Límites y validación

| Regla | Valor MVP |
|---|---|
| Dimensiones | Potencia de dos, ≥ 4 en ambos ejes, múltiplo de 4 (bloque BCn) |
| Máximo por eje | 4096 |
| Máximo de mips | 16 |
| Payload por mip | ≤ `0xFFFFFF` bytes |
| Redimensionado | **Prohibido**. La CLI/lib rechazan no-POT; no pad/resize implícito |
| LZO | No escrito (bit 15 de width = 0). Lectura documentada para futuro `#13` |

## Política de formato y alfa

| Situación | Resultado |
|---|---|
| `format = Auto`, todos los píxeles con A = 255 | DXT1 |
| `format = Auto`, cualquier A < 255 | DXT5 |
| `format = Dxt5` | DXT5 (siempre `GALF=1`) |
| `format = Dxt1`, imagen opaca | DXT1 |
| `format = Dxt1`, hay A < 255 | **Error** (no degradar a 1-bit en silencio). Umbral de transparencia BC1 documentado abajo sólo aplica si en el futuro se añade un modo explícito `dxt1-binary` |

**Semántica BC1 (cuando se usa DXT1):** canal alfa del bloque BC1 de 1 bit;
píxeles se tratan como opacos en el encoder (A se ignora porque la política
exige opacidad previa).

**Semántica BC3:** alfa interpolado de 8 valores; `weigh_colour_by_alpha`
activado en el backend para no contaminar bordes con RGB bajo A=0.

## Mipmaps

- Se generan desde la imagen base **sin mutarla**.
- Filtro: promedio 2×2 en espacio lineal sRGB con **premultiplicación por alfa**,
  luego un-premultiply seguro (A=0 → RGB=0) y vuelta a sRGB.
- Cadena: `(w,h), (w/2,h/2), …` mientras `min(w,h) > 4`; se **incluye** el
  nivel donde `min == 4`, luego se detiene.
- `generate_mips = false` / `--no-mips`: un solo nivel (el base).
- Rectangular: p.ej. `1024×2048` → 9 niveles hasta `4×8`.
- Cuadrado `512×512` → 8 niveles hasta `4×4`.

## Matriz de casos observables

| Caso | Formato | Mips | Tags | Notas |
|---|---|---|---|---|
| Opaco sólido / gradiente | DXT1 | cadena | CGVA, CXAM, SFFO | sin GALF |
| Alfa binaria 0/255 | DXT5 (auto) | cadena | + GALF=1 | silueta dura |
| Alfa suave radial | DXT5 | cadena | + GALF=1 | UI RMTFAR |
| RGB negro bajo A=0 | DXT5 | cadena | + GALF=1 | sin halo al componer |
| Rectangular 8×16 | DXT5 | hasta 4×8 | + GALF=1 | |
| `--no-mips` | según auto/force | 1 | SFFO[0] sólo | |
| `--format dxt1` opaco | DXT1 | … | sin GALF | |
| `--format dxt1` con alfa | error CLI/lib | — | — | exit ≠ 0 |
| `--format dxt5` opaco | DXT5 | … | GALF=1 | |

## Comportamiento CLI (contrato)

- Entrada: PNG o TGA; se carga como RGBA8.
- Salida: debe terminar en `.paa` (o se rechaza).
- Sin `--force`, no sobrescribe.
- Escritura atómica (temp + rename) en el mismo directorio.
- Códigos: `0` ok; `2` uso/flags conflictivos; `1` otros errores.

## Extensiones futuras vs MVP

Cualquier PR que añada LZO, swizzle, formatos no-DXT o tags extra debe
actualizar este perfil en la misma PR y marcar la sección correspondiente.
Hasta entonces, esos caminos no existen en la API pública.
