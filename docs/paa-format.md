# Formato PAA (notas de implementación)

Referencias públicas / ingeniería inversa comunes en la comunidad Arma:

- Contenedor con tablas de mipmaps y tipo de compresión (DXT1 / DXT5 / …).
- Dimensiones casi siempre potencia de dos.
- UI con silueta transparente → **DXT5** (o equivalente con canal alfa).
- Validar siempre en **TexView 2** sobre fondos claros y oscuros (halos).

## Roadmap del encoder

1. [x] CLI + carga PNG/TGA + guardas POT
2. [ ] Compresión de bloques DXT1 / DXT5
3. [ ] Cabecera / tags PAA reconocidos por TexView2 y el motor
4. [ ] Cadena de mipmaps
5. [ ] Tests de regresión contra fixtures conocidos (hashes / round-trip visual)
6. [ ] Integración opcional desde `rmtfar/scripts/convert-radio-textures.sh`

Hasta completar 2–4, el binario escribe un stub marcado `TODO_REAL_PAA` para
no confundir artefactos a medio hacer con texturas jugables.
