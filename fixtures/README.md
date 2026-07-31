# Fixtures

Synthetic PNG sources under `sources/` are original to this project. They contain
**no** Bohemia Interactive, TFAR, ACRE, or third-party artwork.

## Regeneration

```bash
python3 scripts/generate-fixtures.py
```

## Oracles

### Official ImageToPAA.exe (opt-in)

```bash
export IMAGETOPAA_PATH=/path/to/ImageToPAA.exe
./scripts/regenerate-oracle.sh
```

Until redistribution of those derived `.paa` binaries is confirmed, prefer
hashes in `oracle/manifest.template.json` over committing BI-tool outputs.

### Gruppe Adler web converter (checked in)

`oracle/gruppe_adler/` holds one PAA derived from **original RMTFAR** art via
https://paa.gruppe-adler.de/ — used to lock container/tag/mip/LZO expectations
(see README there). Useful for issue #12 / future #13; not a BCn golden file.

## Redistribution policy

- Do **not** commit `ImageToPAA.exe`, TexView, or Arma/TFAR/ACRE game assets.
- Community-converter outputs of **our own** masters may be versioned as
  interoperability samples with clear provenance.
- Bohemia-tool outputs: hashes/metadata first; binaries only when policy allows.
