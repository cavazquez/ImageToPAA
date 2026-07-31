# Oracle: Gruppe Adler online PAA converter

Sample produced outside this repo to validate **container layout** against a
third-party encoder (not Bohemia ImageToPAA.exe).

| Field | Value |
|-------|--------|
| Tool | [Gruppe Adler PAA converter](https://paa.gruppe-adler.de/) (web) |
| Date | 2026-07-31 |
| Input | RMTFAR original `r110_independent_icon_paa_source.png` (512×512 RGBA) |
| Output | `r110_independent_icon.paa` |
| SHA-256 | `45566b655c6fe2389955764051726308a4bd0085229f7afa89178d757262399b` |

## What this oracle confirms

- Type `05 FF` (DXT5), tags `CGVA → CXAM → GALF → SFFO`, empty palette.
- Mip chain 512→4 (8 levels), same stop rule as our MVP profile.
- **LZO** on large mips (`width | 0x8000`) when it shrinks the payload — relevant to ImageToPAA **#13**.
- `GALF` payload here is `01 FF FF FF` (not our MVP `01 00 00 00`); treat as tool variance until more samples exist.
- BCn bytes need not match ours; structural dims/tags are the contract.

## Policy

The PNG source is original RMTFAR art (not BI/TFAR). The derived `.paa` is
checked in as an interoperability sample. Do **not** commit the web tool itself
or any Bohemia/TFAR assets.

Zip drop used to import this sample (monorepo root, not versioned here):
`ProyectoRmtfar/gruppe_adler_paa.zip`.
