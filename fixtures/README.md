# Fixtures

Synthetic PNG sources under `sources/` are original to this project. They contain
**no** Bohemia Interactive, TFAR, ACRE, or third-party artwork.

## Regeneration

```bash
python3 scripts/generate-fixtures.py
```

## Official oracle (opt-in)

```bash
export IMAGETOPAA_PATH=/path/to/ImageToPAA.exe
./scripts/regenerate-oracle.sh
```

## Redistribution policy for oracle outputs

ImageToPAA.exe is proprietary. Until we confirm that **derived `.paa` binaries**
produced by it may be redistributed under this project's license, this repo
commits only:

- synthetic PNG sources;
- `oracle/manifest.template.json` (schema) and any filled `manifest.json` that
  stores **hashes / interpreted headers / tool options**, not the `.paa` bytes;
- optional PNG decodes from TexView when those are clearly our synthetic content.

Do **not** commit `ImageToPAA.exe`, TexView, or Arma game assets.
