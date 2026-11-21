//! High-resolution MIDI output and clip-roll scheduling.

use midir::{MidiOutput, MidiOutputConnection};
use openvj_core::{ClipId, MidiRoll, MIDI_TICKS_PER_BEAT};
use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

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

    fn open(&mut self, requested: &str) -> Result<(), String> {
        if self.connection.is_some() && self.device == requested { return Ok(()); }
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

    fn close(&mut self) {
        self.all_notes_off();
        self.connection = None;
        self.device.clear();
    }

    fn note_on(&mut self, channel: u8, pitch: u8, velocity: u8) {
        self.send(&[0x90 | channel.min(15), pitch.min(127), velocity.clamp(1, 127)]);
    }

    fn note_off(&mut self, channel: u8, pitch: u8) {
        self.send(&[0x80 | channel.min(15), pitch.min(127), 0]);
    }

    fn all_notes_off(&mut self) {
        for channel in 0..16u8 { self.send(&[0xB0 | channel, 123, 0]); }
    }

    fn send(&mut self, message: &[u8]) {
        if let Some(connection) = self.connection.as_mut() { let _ = connection.send(message); }
    }
}

#[derive(Clone)]
pub struct MidiRollSnapshot {
    pub id: ClipId,
    pub roll: MidiRoll,
    pub started_beat: f64,
    pub speed: f32,
}

enum OutputCommand {
    Sync { rolls: Vec<MidiRollSnapshot>, beat: f64, bpm: f32 },
    Preview { device: String, channel: u8, pitch: u8, velocity: u8 },
    AllNotesOff,
    Shutdown,
}

struct RollState {
    roll: MidiRoll,
    device: String,
    channel: u8,
    started_beat: f64,
    speed: f32,
    last_tick: f64,
    active: HashMap<u8, f64>,
}

enum Event { On(u8, u8, u8), Off(u8, u8) }

/// MIDI scheduling is isolated from the UI/render thread. The worker advances
/// the beat between App::frame snapshots, giving short notes usable timing even
/// when the VJ render cadence is 30 FPS.
pub struct MidiOutputWorker {
    tx: Sender<OutputCommand>,
    join: Option<JoinHandle<()>>,
}

impl MidiOutputWorker {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        let join = thread::Builder::new().name("syntetika-midi-scheduler".into()).spawn(move || run(rx)).expect("MIDI scheduler thread");
        Self { tx, join: Some(join) }
    }

    pub fn sync(&self, rolls: Vec<MidiRollSnapshot>, beat: f64, bpm: f32) {
        let _ = self.tx.send(OutputCommand::Sync { rolls, beat, bpm });
    }

    pub fn all_notes_off(&self) { let _ = self.tx.send(OutputCommand::AllNotesOff); }

    /// Auditions a short note on a device, used by the Piano Roll keyboard.
    pub fn preview(&self, device: &str, channel: u8, pitch: u8, velocity: u8) {
        let _ = self.tx.send(OutputCommand::Preview {
            device: device.to_owned(),
            channel,
            pitch,
            velocity,
        });
    }
}

impl Drop for MidiOutputWorker {
    fn drop(&mut self) {
        let _ = self.tx.send(OutputCommand::AllNotesOff);
        let _ = self.tx.send(OutputCommand::Shutdown);
        if let Some(join) = self.join.take() { let _ = join.join(); }
    }
}

fn run(rx: Receiver<OutputCommand>) {
    let mut outputs: HashMap<String, MidiOutputManager> = HashMap::new();
    let mut states: HashMap<ClipId, RollState> = HashMap::new();
    let mut previews: Vec<(Instant, String, u8, u8)> = Vec::new();
    let mut anchor_beat = 0.0;
    let mut anchor_at = Instant::now();
    let mut bpm = 120.0f32;

    loop {
        match rx.recv_timeout(Duration::from_millis(1)) {
            Ok(OutputCommand::Sync { rolls, beat, bpm: next_bpm }) => {
                anchor_beat = beat;
                anchor_at = Instant::now();
                bpm = next_bpm.max(1.0);
                let ids: std::collections::HashSet<_> = rolls.iter().map(|r| r.id).collect();
                let stale: Vec<_> = states.keys().copied().filter(|id| !ids.contains(id)).collect();
                for id in stale { stop_state(&mut states, &mut outputs, id); }
                let current = beat_now(anchor_beat, anchor_at, bpm);
                for snapshot in rolls {
                    if !snapshot.roll.enabled || snapshot.roll.notes.is_empty() || snapshot.roll.output_device.is_empty() {
                        stop_state(&mut states, &mut outputs, snapshot.id);
                        continue;
                    }
                    let channel = snapshot.roll.normalized_channel();
                    let replace = states.get(&snapshot.id).is_some_and(|state| {
                        state.started_beat != snapshot.started_beat || state.device != snapshot.roll.output_device || state.channel != channel
                    });
                    if replace { stop_state(&mut states, &mut outputs, snapshot.id); }
                    states.entry(snapshot.id).or_insert_with(|| RollState {
                        roll: snapshot.roll.clone(),
                        device: snapshot.roll.output_device.clone(),
                        channel,
                        started_beat: snapshot.started_beat,
                        speed: snapshot.speed,
                        last_tick: midi_tick(current, snapshot.started_beat, snapshot.speed) - 0.001,
                        active: HashMap::new(),
                    });
                    if let Some(state) = states.get_mut(&snapshot.id) {
                        state.roll = snapshot.roll;
                        state.speed = snapshot.speed;
                    }
                }
            }
            Ok(OutputCommand::Preview { device, channel, pitch, velocity }) => {
                let output = outputs.entry(device.clone()).or_default();
                if output.open(&device).is_ok() {
                    output.note_on(channel.min(15), pitch.min(127), velocity);
                    previews.push((Instant::now(), device, channel.min(15), pitch.min(127)));
                }
            }
            Ok(OutputCommand::AllNotesOff) => {
                for output in outputs.values_mut() { output.all_notes_off(); }
                states.clear();
            }
            Ok(OutputCommand::Shutdown) | Err(RecvTimeoutError::Disconnected) => break,
            Err(RecvTimeoutError::Timeout) => {}
        }

        let now = Instant::now();
        if !previews.is_empty() {
            previews.retain(|(at, device, channel, pitch)| {
                if now.duration_since(*at) >= Duration::from_millis(180) {
                    if let Some(output) = outputs.get_mut(device) {
                        output.note_off(*channel, *pitch);
                    }
                    false
                } else {
                    true
                }
            });
        }
        let current_beat = beat_now(anchor_beat, anchor_at, bpm);        let ids: Vec<_> = states.keys().copied().collect();
        for id in ids {
            process_state(&mut states, &mut outputs, id, current_beat);
        }
    }

    for output in outputs.values_mut() { output.close(); }
}

fn beat_now(anchor: f64, at: Instant, bpm: f32) -> f64 {
    anchor + at.elapsed().as_secs_f64() * bpm as f64 / 60.0
}

fn midi_tick(beat: f64, started: f64, speed: f32) -> f64 {
    (beat - started).max(0.0) * speed.max(0.0) as f64 * MIDI_TICKS_PER_BEAT as f64
}

fn stop_state(states: &mut HashMap<ClipId, RollState>, outputs: &mut HashMap<String, MidiOutputManager>, id: ClipId) {
    if let Some(state) = states.remove(&id) {
        if let Some(output) = outputs.get_mut(&state.device) {
            for pitch in state.active.keys().copied().collect::<Vec<_>>() { output.note_off(state.channel, pitch); }
        }
    }
}

fn process_state(states: &mut HashMap<ClipId, RollState>, outputs: &mut HashMap<String, MidiOutputManager>, id: ClipId, beat: f64) {
    let Some(state) = states.get_mut(&id) else { return };
    if state.device.is_empty() || !state.roll.enabled || state.roll.notes.is_empty() { return; }
    let device = state.device.clone();
    let output = outputs.entry(device.clone()).or_default();
    if let Err(error) = output.open(&device) { eprintln!("Syntetika MIDI output: {error}"); return; }

    let current = midi_tick(beat, state.started_beat, state.speed);
    let events = collect_events(state, current);
    for event in events {
        match event { Event::On(ch, p, v) => output.note_on(ch, p, v), Event::Off(ch, p) => output.note_off(ch, p) }
    }
}

fn collect_events(state: &mut RollState, current: f64) -> Vec<Event> {
    let loop_start = state.roll.loop_start as f64;
    let loop_end = state.roll.loop_end_for_bars() as f64;
    let loop_len = (loop_end - loop_start).max(1.0);
    if current < state.last_tick {
        state.active.clear();
        state.last_tick = current - 0.001;
    }
    let previous = state.last_tick;
    let first_cycle = (previous / loop_len).floor() as i64;
    let last_cycle = (current / loop_len).floor() as i64;
    let mut events = Vec::new();

    for note in state.roll.normalized_notes() {
        let start = note.start_tick as f64;
        if start < loop_start || start >= loop_end { continue; }
        let relative = start - loop_start;
        for cycle in first_cycle..=last_cycle {
            let event = cycle as f64 * loop_len + relative;
            if event > previous && event <= current {
                if state.active.remove(&note.pitch).is_some() { events.push(Event::Off(state.channel, note.pitch)); }
                events.push(Event::On(state.channel, note.pitch, note.velocity));
                state.active.insert(note.pitch, event + note.length_ticks.max(1) as f64);
            }
        }
    }
    let finished: Vec<_> = state.active.iter().filter_map(|(pitch, end)| (*end <= current).then_some(*pitch)).collect();
    for pitch in finished {
        state.active.remove(&pitch);
        events.push(Event::Off(state.channel, pitch));
    }
    state.last_tick = current;
    events
}

#[cfg(test)]
mod tests {
    use super::*;
    use openvj_core::{MidiNote, MidiRoll};

    fn state(notes: Vec<MidiNote>, last_tick: f64) -> RollState {
        RollState {
            roll: MidiRoll { enabled: true, notes, ..MidiRoll::default() },
            device: String::new(),
            channel: 2,
            started_beat: 0.0,
            speed: 1.0,
            last_tick,
            active: HashMap::new(),
        }
    }

    #[test]
    fn short_note_emits_note_on_at_tick_boundary() {
        let mut state = state(vec![MidiNote { pitch: 60, start_tick: 0, length_ticks: 2, velocity: 100 }], -0.001);
        assert!(matches!(collect_events(&mut state, 0.01).as_slice(), [Event::On(2, 60, 100)]));
    }

    #[test]
    fn note_off_is_emitted_when_length_expires() {
        let mut state = state(vec![MidiNote { pitch: 60, start_tick: 0, length_ticks: 2, velocity: 100 }], -0.001);
        let _ = collect_events(&mut state, 0.01);
        assert!(matches!(collect_events(&mut state, 2.0).as_slice(), [Event::Off(2, 60)]));
    }

    #[test]
    fn loop_boundary_emits_the_next_note() {
        let mut state = state(vec![MidiNote { pitch: 64, start_tick: 0, length_ticks: 4, velocity: 90 }], 127.9);
        assert!(matches!(collect_events(&mut state, 128.1).as_slice(), [Event::On(2, 64, 90)]));
    }

    #[test]
    fn rewinding_resets_active_notes_without_duplicate_state() {
        let mut state = state(vec![MidiNote { pitch: 67, start_tick: 16, length_ticks: 4, velocity: 80 }], 20.0);
        state.active.insert(67, 24.0);
        let events = collect_events(&mut state, 4.0);
        assert!(events.is_empty());
        assert!(state.active.is_empty());
    }
}
