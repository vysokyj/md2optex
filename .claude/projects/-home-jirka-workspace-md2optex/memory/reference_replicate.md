---
name: Replicate API for image generation
description: Replicate account set up with FLUX.1 schnell model for generating images via curl
type: reference
---

Jiří má Replicate účet s API tokenem v `~/.zshrc` (`REPLICATE_API_TOKEN`). Platební karta přidána.

Model: `black-forest-labs/flux-schnell` — levný (~$0.003/obrázek), rychlý, dobrá kvalita.

Volání přes curl (ne Python SDK — není nainstalovaný):
```bash
source ~/.zshrc
curl -s -X POST "https://api.replicate.com/v1/models/black-forest-labs/flux-schnell/predictions" \
  -H "Authorization: Bearer $REPLICATE_API_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"input":{"prompt":"...","num_outputs":1}}'
```

Polling výsledku: `GET /v1/predictions/{id}`, output v `.output[0]` jako URL na webp.
