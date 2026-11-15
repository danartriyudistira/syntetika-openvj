## 1. Proposal

- [x] 1.1 Proposal, design and spec delta (this PR, first)

## 2. Engine

- [ ] 2.1 Add `lactea` (pinned) to openvj-render; credit it in THIRD_PARTY_NOTICES.md
- [ ] 2.2 `ClipSource::Milkdrop` in openvj-core, saved by preset path; Sensitivity parameter
- [ ] 2.3 One lactea engine per MilkDrop clip, rendered once per frame and shared by layer, preview and thumbnail
- [ ] 2.4 Feed raw audio samples from the app to the MilkDrop engines
- [ ] 2.5 Tests: a preset renders non-black, reacts to a tone, a missing preset draws transparent

## 3. App

- [ ] 3.1 MilkDrop folder scan with hot reload (`~/Documents/openVJ/MilkDrop`, `OPENVJ_MILKDROP_DIR`)
- [ ] 3.2 Sources tab lists presets; `.milk` files from the Files tab place as MilkDrop clips
- [ ] 3.3 Clip panel: description, missing/error message, Sensitivity
- [ ] 3.4 Smoke-test a few hundred presets from the projectM packs for load errors and frame time
- [ ] 3.5 README "What works"; archive the change
