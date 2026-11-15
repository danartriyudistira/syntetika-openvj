# Proposal

## Why

openVJ has one screen today, built for mixing a full composition the way Resolume Arena does. The maintainer also wants to use openVJ in two lighter ways: leaving it running as an audio-reactive visualiser that plays a playlist by itself, and sitting with one generator or layer to shape and play it live. openDJ already switches between whole-window layouts (Prep, Player, 4-Deck, 2-Deck) from a dropdown in the top bar, and openVJ should feel the same.

## What Changes

- A **layout selector** at the left of the top bar, styled like openDJ's: a button with the current layout's icon and name, opening a menu titled LAYOUT with one row per layout (icon + name) and a check mark on the current one. Cyan accent instead of openDJ's lilac. openDJ's pencil and Dark/Light/Auto row are left out (openVJ has one theme).
- Three layouts:
  - **Mixer**: today's screen, unchanged.
  - **Player**: a very simple automix player, like openDJ's Automix. It plays a **playlist** of **presets** one after another by itself, with a big output preview. A preset is a generator (or a clip) with its parameter values, audio mappings and effects, so the same generator can appear several times with different looks, as in Synesthesia. Each preset plays for a set number of bars (or seconds), then the next one comes in with a transition set by LENGTH (time or bars, as in openDJ) and STYLE (crossfade, cut, …). Shuffle and Repeat as in openDJ. Playlists save as their own files, so they can be prepared before a gig and loaded there.
  - **Synthesis**: a screen for one visual at a time: a big preview next to large parameter bars for the selected generator or layer and its effects, made to be played live (by mouse and MIDI). Exact contents are open questions below.
- Switching layout changes only what the window shows. Output, Syphon, mapping, audio, tempo and MIDI keep running, and nothing in the composition changes.
- The app remembers the last layout and opens in it.
- Keyboard shortcuts Cmd+1, Cmd+2, Cmd+3 (Ctrl on Linux and Windows) switch layout, as in openDJ.

Out of scope: user-defined or editable layouts, a light theme, and changes to the Mixer screen itself.

## Capabilities

### New Capabilities
- `layouts`: choosing between the Mixer, Player and Synthesis layouts, and what each shows.

### Modified Capabilities
- None. `visual-style` already covers menus and the accent.

## Impact

The selector and Mixer are UI only, plus a small saved preference. The Player adds presets, a playlist file and its own playback alongside the composition; see design.md.

## Touches

- crates/openvj-app: `ui/main.slint` (top bar `Toolbar`, `MainWindow` body split into one component per layout), new `ui/layouts.slint` (selector, Player, Synthesis), `ui/controls.slint` (menu row, layout icons), `ui/theme.slint` (tokens only if needed), `src/main.rs` and `src/app.rs` (layout property, shortcuts, saving the preference)
- crates/openvj-core: new `playlist.rs` (presets and the playlist file); no change to `composition.rs` beyond reusing its clip, parameter and effect types
- crates/openvj-render: playing the Player's two layers and the transition, without touching the composition's layers
- Specs: new `layouts`
- Shared hot spots: `ui/main.slint`, `ui/theme.slint`, `src/app.rs`

## Conflicts

- The "Remove macOS title bar" thread (no PR yet) makes openVJ's black top bar the window's top edge and moves things in the same `Toolbar`. Suggested order: merge the title-bar PR first, then rebase this one onto it, since the selector simply sits at the left of whatever bar it leaves (to the right of the macOS window buttons).
- add-parameter-animation: no shared files at proposal stage. If Synthesis reuses the parameter rows it restyles, whichever lands second adapts.

## Open questions

For the maintainer:

Answered 2026-11-15: playlists hold presets (generator + settings + effects, clips maybe), several presets of the same generator allowed; timing and transitions use openDJ's LENGTH and STYLE; playlists save as their own files. Reactivity: Seba unsure, so the default in design.md applies (each preset keeps its own audio mappings; the existing Reactivity control scales them all).

Still open:

1. **Synthesis, scope.** One generator or layer at a time (picked from a list), or all layers visible at once like a synth's patch panel?
2. **Synthesis, "play".** What does playing the visuals mean: big parameter bars to ride with the mouse and MIDI, XY pads, trigger/bang buttons, a randomise button, snapshots of parameter values to jump between, or all of these?
3. **Synthesis, clip grid.** Hidden entirely, or a small strip to pick the next clip?
4. **Synthesis and presets.** Should Synthesis be where presets are made ("Save preset" / "Add to playlist"), so a look shaped there can go straight into a Player playlist?
