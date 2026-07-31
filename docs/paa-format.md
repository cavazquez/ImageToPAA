# Formato PAA — índice

El contrato normativo del MVP DXT está en **[`paa-profile.md`](paa-profile.md)**.

Este archivo sólo apunta al perfil y resume el estado del encoder.

## Estado

1. [x] CLI + carga PNG/TGA + guardas POT
2. [x] Compresión BC1 / BC3 (`texpresso`)
3. [x] Cabecera / tags PAA (`CGVA`, `CXAM`, `GALF`, `SFFO`)
4. [x] Cadena de mipmaps alpha-correct / sRGB
5. [x] Fixtures sintéticos + manifiesto
6. [x] Oráculo estructural Gruppe Adler (fixtures/oracle/)
7. [x] LZO por mip opcional (`--compress`, issue #13)
8. [ ] Oráculo oficial ImageToPAA.exe (opt-in, ver `scripts/`)
