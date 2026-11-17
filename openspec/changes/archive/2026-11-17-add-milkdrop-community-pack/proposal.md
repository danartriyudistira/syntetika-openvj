# Proposal

## Why

openVJ plays MilkDrop presets but ships none, because community presets carry no licence. The maintainer wants the community pack available without bundling it (2026-11-17): ask the VJ once whether to add it, and do nothing if they say no.

## What Changes

- On first launch, when the MilkDrop folder has no presets, a dialog offers projectM's "Cream of the Crop" pack (about 9,800 presets) plus the MilkDrop texture pack. **Add the pack** downloads both from GitHub into `~/Documents/openVJ/MilkDrop` (`Cream of the Crop/` and `textures/`); **No thanks** is remembered and the dialog doesn't come back on its own.
- The Sources tab's **MilkDrop…** button opens the same dialog any time (download, download again, open folder).
- Progress shows in the dialog; the result shows in the status bar. The folder's hot reload lists the presets when they're unpacked.
- Still nothing bundled in the app or the repo.

Out of scope: other packs, choosing a subset, search and favourites (the library covers browsing).

## Capabilities

### Modified Capabilities
- `media-sources`: the MilkDrop community pack, offered as a download.

## Touches

- crates/openvj-app: new `milk_pack.rs`, new `ui/milkpack.slint`, `main.rs` (dialog wiring), `app.rs` (progress), `ui/main.slint` (dialog, MilkDrop… button), `Cargo.toml` (`zip`).
- README.md, specs: media-sources.
- Shared hot spots: app.rs, main.slint, README.md (small additions).

## Conflicts

None known: no other open PRs.

## Open questions

None.
