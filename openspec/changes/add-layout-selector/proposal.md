# Proposal

## Why

openVJ has one screen today, built for mixing a full composition the way Resolume Arena does. The maintainer also wants to use openVJ in two lighter ways: leaving it running as an audio-reactive visualiser that plays a playlist by itself, and sitting with one generator or layer to shape and play it live. openDJ already switches between whole-window layouts (Prep, Player, 4-Deck, 2-Deck) from a dropdown in the top bar, and openVJ should feel the same.

## What Changes

- A **layout selector** at the left of the top bar, styled like openDJ's: a button with the current layout's icon and name, opening a menu titled LAYOUT with one row per layout (icon + name) and a check mark on the current one. Cyan accent instead of openDJ's lilac. openDJ's pencil and Dark/Light/Auto row are left out (openVJ has one theme).
- Three layouts:
  - **Mixer**: today's screen, unchanged.
  - **Player**: a very simple automix player. A playlist of visuals plays one after another by itself, reacting to the audio input, with a big output preview and a few transport controls. Exact contents are open questions below.
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

UI only, plus a small saved preference. The Player's playlist may need a new saved model (see Open questions); if so it gets its own design.md before code.

## Touches

- crates/openvj-app: `ui/main.slint` (top bar `Toolbar`, `MainWindow` body split into one component per layout), new `ui/layouts.slint` (selector, Player, Synthesis), `ui/controls.slint` (menu row, layout icons), `ui/theme.slint` (tokens only if needed), `src/main.rs` and `src/app.rs` (layout property, shortcuts, saving the preference)
- Specs: new `layouts`
- Shared hot spots: `ui/main.slint`, `ui/theme.slint`, `src/app.rs`

## Conflicts

- The "Remove macOS title bar" thread (no PR yet) makes openVJ's black top bar the window's top edge and moves things in the same `Toolbar`. Suggested order: merge the title-bar PR first, then rebase this one onto it, since the selector simply sits at the left of whatever bar it leaves (to the right of the macOS window buttons).
- add-parameter-animation: no shared files at proposal stage. If Synthesis reuses the parameter rows it restyles, whichever lands second adapts.

## Open questions

For the maintainer:

1. **Player, what is in the playlist?** ISF generators (incl. Ghost Arcade), video files, whole decks/columns of the current composition, or saved compositions? Suggestion: generators and video files from the browser, dragged in.
2. **Player, when does it move on?** Every N bars (suggest 16, following BPM/Link), on a timer in seconds, or on musical changes (drops, breakdowns, like Resolume's Autopilot)? And how: crossfade of a set length in beats, or cut?
3. **Player, audio-reactive how?** Rely on each visual's own audio mappings, or have the Player add an overall Reactivity and some automatic modulation so any visual moves with the music?
4. **Player, extras?** Shuffle, loop playlist, effects on the whole output, and is the playlist saved with the composition or on its own as a file?
5. **Synthesis, scope.** One generator or layer at a time (picked from a list), or all layers visible at once like a synth's patch panel?
6. **Synthesis, "play".** What does playing the visuals mean to you: big parameter bars to ride with the mouse and MIDI, XY pads, trigger/bang buttons, a randomise button, snapshots of parameter values to jump between, or all of these?
7. **Synthesis, clip grid.** Hidden entirely, or a small strip to pick the next clip?
