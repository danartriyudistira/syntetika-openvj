## ADDED Requirements

### Requirement: MilkDrop community pack
openVJ SHALL offer to download projectM's "Cream of the Crop" MilkDrop presets and the MilkDrop texture pack into the MilkDrop folder. It SHALL ask once, on a launch where the MilkDrop folder has no presets, and SHALL NOT ask again after the VJ declines. The Sources tab SHALL open the same offer on request.

#### Scenario: Accept on first launch
- **GIVEN** a fresh install with an empty MilkDrop folder
- **WHEN** openVJ starts and the VJ chooses Add the pack
- **THEN** the presets download into `MilkDrop/Cream of the Crop` with their textures, and appear in the library when unpacked

#### Scenario: Decline
- **WHEN** the VJ chooses No thanks
- **THEN** nothing is downloaded and the offer doesn't open on later launches

#### Scenario: Later
- **WHEN** the VJ clicks MilkDrop… in the Sources tab
- **THEN** the offer opens again, with the folder button
