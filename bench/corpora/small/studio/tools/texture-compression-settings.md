---
name: texture-compression-settings
description: Texture compression by usage suffix: albedo BC7, normal BC5, masks BC4, HDR BC6H, quality levels, BC1 dropped in 2026
type: reference
status: active
verified: 2026-03-05
---

The pipeline ([[asset-pipeline-overview]]) picks the compression from the texture's name suffix, which is the naming rule the common conventions impose. No per-texture override in a metadata file; if a texture needs something else, its suffix is wrong.

| Suffix | Usage | Desktop | Low-end | Mips |
|---|---|---|---|---|
| `_alb` | albedo, sRGB | BC7 | BC7 | yes |
| `_nrm` | normal map, 2 channels reconstructed | BC5 | proprietary 2-channel | yes |
| `_msk` | single-channel mask (roughness, AO, height) | BC4 | BC4 | yes |
| `_orm` | packed occlusion, roughness, metallic | BC7 | BC7 | yes |
| `_hdr` | HDR (skies, probes) | BC6H | BC6H | yes |
| `_ui` | UI, no mips, sRGB | BC7 | BC7 | no |
| `_raw` | uncompressed, tools only | RGBA8 | not shipped | no |

## Quality

BC7 encoding runs at the "high" level of the encoder on the CI (24 minutes of the 41-minute full build) and at "fast" on dev machines, which is why a texture can look slightly different locally and in a CI build; the diff is under the visible threshold on the 40 reference captures but artists asked, so it is written here.

## BC1 dropped

Until January 2026 albedos without alpha used BC1 to save half the memory. Banding on the sea and sky gradients was visible on the low-end target's screen, and the memory saved (about 180 MB) was found instead by the mip eviction of the streamer. Every `_alb` is BC7 now; the pipeline refuses `BC1` in any setting.

## Maximum sizes

4096 on desktop, 2048 on the low-end target (the pipeline drops the top mip), 1024 for anything with suffix `_msk` unless the file name contains `_hi`. A source larger than 8192 fails the import with `texture.too_large`, see [[asset-import-failures-2026]].
