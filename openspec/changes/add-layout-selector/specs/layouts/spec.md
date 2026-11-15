## ADDED Requirements

### Requirement: Layout selector
The top bar SHALL show, at its left, a layout selector like openDJ's: a button with the current layout's icon and name that opens a menu titled LAYOUT, listing Mixer, Player and Synthesis with an icon each and a check mark in the cyan accent on the current one.

#### Scenario: Switch layout from the menu
- **WHEN** the VJ opens the layout menu and picks Synthesis
- **THEN** the window shows the Synthesis layout and the button reads Synthesis

#### Scenario: Switch layout from the keyboard
- **WHEN** the VJ presses Cmd+2 (Ctrl+2 on Linux and Windows)
- **THEN** the window shows the Player layout

### Requirement: Switching keeps the show running
Switching layout SHALL change only what the main window shows. Output, Syphon, mapping, audio input, tempo, Link, MIDI and the composition SHALL keep running unchanged.

#### Scenario: Switch during a show
- **WHEN** clips are playing to the output and the VJ switches from Mixer to Player and back
- **THEN** the output never drops a frame to black and the same clips are still playing

### Requirement: Remembered layout
openVJ SHALL remember the last layout used and open in it next time. A first launch opens in Mixer.

#### Scenario: Reopen the app
- **WHEN** the VJ quits in Synthesis and opens openVJ again
- **THEN** it opens in Synthesis

### Requirement: Mixer layout
The Mixer layout SHALL be the full mixing screen: clip grid with deck tabs, monitors, master and crossfader, properties dock and browser.

#### Scenario: Default layout
- **WHEN** openVJ opens for the first time
- **THEN** it shows the Mixer layout, the same screen as before layouts existed

### Requirement: Player layout
The Player layout SHALL be a very simple automix player: a playlist of visuals that plays one after another by itself, reacting to the audio input, with a large output preview and play, pause and next controls.

#### Scenario: Leave it running
- **WHEN** the VJ adds visuals to the playlist and presses play
- **THEN** openVJ moves through the playlist on its own, each visual reacting to the music

### Requirement: Synthesis layout
The Synthesis layout SHALL focus on one visual at a time: a large preview beside large parameter bars for the selected generator or layer and its effects, made to be played live by mouse and MIDI.

#### Scenario: Play a generator
- **WHEN** the VJ selects a layer playing a generator in Synthesis
- **THEN** its parameters and effects fill the screen as large bars that change the output as they are moved
