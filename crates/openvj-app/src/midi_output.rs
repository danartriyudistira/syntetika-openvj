//! MIDI output ports used by clip Piano Rolls.

use midir::{MidiOutput, MidiOutputConnection};

const CLIENT: &str = "Syntetika OpenVJ";

pub struct MidiOutputManager {
    connection: Option<MidiOutputConnection>,
    device: String,
}

impl Default for MidiOutputManager {
    fn default() -> Self {
        Self { connection: None, device: String::new() }
    }
}

impl MidiOutputManager {
    pub fn port_names() -> Result<Vec<String>, String> {
        let output = MidiOutput::new(CLIENT).map_err(|e| e.to_string())?;
        Ok(output.ports().iter().filter_map(|p| output.port_name(p).ok()).collect())
    }

    pub fn open(&mut self, requested: &str) -> Result<(), String> {
        if requested.is_empty() {
            self.close();
            return Ok(());
        }
        if self.connection.is_some() && self.device == requested {
            return Ok(());
        }

        self.close();
        let output = MidiOutput::new(CLIENT).map_err(|e| e.to_string())?;
        let port = output
            .ports()
            .into_iter()
            .find(|p| output.port_name(p).is_ok_and(|name| name == requested))
            .ok_or_else(|| format!("MIDI output device not found: {requested}"))?;
        self.connection = Some(output.connect(&port, "syntetika-midi-roll").map_err(|e| e.to_string())?);
        self.device = requested.to_owned();
        Ok(())
    }

    pub fn close(&mut self) {
        self.all_notes_off();
        self.connection = None;
        self.device.clear();
    }

    pub fn device(&self) -> &str { &self.device }

    pub fn note_on(&mut self, channel: u8, pitch: u8, velocity: u8) {
        self.send(&[0x90 | channel.min(15), pitch.min(127), velocity.clamp(1, 127)]);
    }

    pub fn note_off(&mut self, channel: u8, pitch: u8) {
        self.send(&[0x80 | channel.min(15), pitch.min(127), 0]);
    }

    pub fn all_notes_off(&mut self) {
        for channel in 0..16u8 {
            self.send(&[0xB0 | channel, 123, 0]);
        }
    }

    fn send(&mut self, message: &[u8]) {
        if let Some(connection) = self.connection.as_mut() {
            let _ = connection.send(message);
        }
    }
}

impl Drop for MidiOutputManager {
    fn drop(&mut self) {
        self.close();
    }
}
