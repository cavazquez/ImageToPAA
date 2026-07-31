# Formato PAA — índice

El contrato normativo del MVP DXT está en **[`paa-profile.md`](paa-profile.md)**.

Este archivo sólo apunta al perfil y resume el estado del encoder.

## Estado

1. [x] CLI + carga PNG/TGA + guardas POT
2. [x] Compresión BC1 / BC3 (`texpresso`)
3. [x] Cabecera / tags PAA (`CGVA`, `CXAM`, `GALF`, `SFFO`)
4. [x] Cadena de mipmaps alpha-correct / sRGB
5. [x] Fixtures sintéticos + manifiesto
6. [ ] Oráculo oficial ImageToPAA (opt-in, ver `scripts/` y issue #12)
7. [ ] LZO por mip (issue #13)
