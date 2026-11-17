//! The MilkDrop community pack: projectM's "Cream of the Crop" presets and the MilkDrop
//! texture pack, downloaded on request into the openVJ MilkDrop folder.
//!
//! openVJ doesn't ship MilkDrop presets (most have no stated licence), so the first launch
//! asks once whether to fetch them (ui/milkpack.slint). The download runs on a background
//! thread; [`progress`] reports how far it got, and the MilkDrop folder's hot reload lists
//! the presets as soon as they're unpacked.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// The presets: about 9,800 curated by ISOSCELES for projectM.
pub const PRESETS_URL: &str =
    "https://github.com/projectM-visualizer/presets-cream-of-the-crop/archive/refs/heads/master.zip";
/// The textures many presets sample (`sampler_clouds2` and so on).
pub const TEXTURES_URL: &str =
    "https://github.com/projectM-visualizer/presets-milkdrop-texture-pack/archive/refs/heads/master.zip";
/// Where the presets go inside the MilkDrop folder.
pub const PACK_FOLDER: &str = "Cream of the Crop";
/// The preference that remembers the VJ answered the first-launch question.
pub const ASKED_FLAG: &str = "milkdrop_pack_asked";

/// How a download is going.
#[derive(Debug, Clone, PartialEq)]
pub enum Progress {
    Idle,
    /// Step label ("Downloading presets"), bytes so far, total bytes when known.
    Running {
        step: String,
        done: u64,
        total: Option<u64>,
    },
    Done {
        presets: usize,
    },
    Failed(String),
}

static STATE: Mutex<Progress> = Mutex::new(Progress::Idle);

pub fn progress() -> Progress {
    STATE.lock().map(|s| s.clone()).unwrap_or(Progress::Idle)
}

fn set(p: Progress) {
    if let Ok(mut s) = STATE.lock() {
        *s = p;
    }
}

pub fn is_running() -> bool {
    matches!(progress(), Progress::Running { .. })
}

/// Whether the first launch should ask: never asked, and no presets in the MilkDrop folder.
pub fn should_ask() -> bool {
    if crate::prefs::flag(ASKED_FLAG) {
        return false;
    }
    let Some(dir) = openvj_core::milkdrop::user_dir() else { return false };
    openvj_core::milkdrop::scan(&[dir]).is_empty()
}

/// Whether the pack is already in the MilkDrop folder.
pub fn installed() -> bool {
    openvj_core::milkdrop::user_dir().is_some_and(|d| d.join(PACK_FOLDER).is_dir())
}

/// Starts downloading the pack in the background. `done` runs on that thread when it ends.
pub fn start(done: impl FnOnce() + Send + 'static) {
    if is_running() {
        return;
    }
    let Some(dir) = openvj_core::milkdrop::user_dir() else {
        set(Progress::Failed("there's no home folder to save into".into()));
        done();
        return;
    };
    set(Progress::Running { step: "Starting".into(), done: 0, total: None });
    let spawned = std::thread::Builder::new().name("milkdrop-pack".into()).spawn(move || {
        set(match install(&dir) {
            Ok(presets) => Progress::Done { presets },
            Err(e) => Progress::Failed(e),
        });
        done();
    });
    if let Err(e) = spawned {
        set(Progress::Failed(e.to_string()));
    }
}

fn install(dir: &Path) -> Result<usize, String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("Couldn't create {}: {e}", dir.display()))?;
    let presets_zip = dir.join(".cream-of-the-crop.zip");
    let textures_zip = dir.join(".textures.zip");
    let result = (|| {
        download(PRESETS_URL, &presets_zip, "Downloading presets")?;
        download(TEXTURES_URL, &textures_zip, "Downloading textures")?;
        set(Progress::Running { step: "Unpacking".into(), done: 0, total: None });
        let staging = dir.join(".cream-of-the-crop.partial");
        let _ = std::fs::remove_dir_all(&staging);
        let presets = unpack(&presets_zip, &staging, is_pack_file, false)?;
        if presets == 0 {
            return Err("the download had no presets in it".into());
        }
        unpack(&textures_zip, &dir.join("textures"), is_texture, true)?;
        // Swap the new pack in whole, so the folder scan never lists half of it.
        let target = dir.join(PACK_FOLDER);
        let old = dir.join(".cream-of-the-crop.old");
        let _ = std::fs::remove_dir_all(&old);
        if target.exists() {
            std::fs::rename(&target, &old).map_err(|e| e.to_string())?;
        }
        std::fs::rename(&staging, &target).map_err(|e| e.to_string())?;
        let _ = std::fs::remove_dir_all(&old);
        Ok(presets)
    })();
    let _ = std::fs::remove_file(&presets_zip);
    let _ = std::fs::remove_file(&textures_zip);
    result
}

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_connect(Some(std::time::Duration::from_secs(20)))
        .user_agent("openVJ")
        .http_status_as_error(false)
        .build()
        .new_agent()
}

/// Streams `url` into `path`, reporting progress under `step`.
fn download(url: &str, path: &Path, step: &str) -> Result<(), String> {
    let mut response = agent().get(url).call().map_err(|e| format!("{step}: {e}"))?;
    if !response.status().is_success() {
        return Err(format!("{step}: GitHub answered {}", response.status()));
    }
    let total = response.body().content_length();
    let mut reader = response.body_mut().with_config().limit(1 << 30).reader();
    let mut file = std::fs::File::create(path).map_err(|e| format!("Couldn't write {}: {e}", path.display()))?;
    let mut buf = vec![0u8; 1 << 16];
    let mut done = 0u64;
    loop {
        let n = reader.read(&mut buf).map_err(|e| format!("{step}: {e}"))?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n]).map_err(|e| e.to_string())?;
        done += n as u64;
        set(Progress::Running { step: step.into(), done, total });
    }
    Ok(())
}

fn ext(path: &Path) -> String {
    path.extension().map(|e| e.to_string_lossy().to_ascii_lowercase()).unwrap_or_default()
}

/// Presets and the pack's licence and readme.
fn is_pack_file(path: &Path) -> bool {
    ext(path) == "milk" || (path.components().count() == 1 && ext(path) == "md")
}

fn is_texture(path: &Path) -> bool {
    matches!(ext(path).as_str(), "jpg" | "jpeg" | "png" | "bmp" | "tga" | "dds")
}

/// Unpacks the files of a GitHub archive that pass `keep` into `into`, dropping the
/// archive's top folder (and every folder when `flat`). Returns how many `.milk` files it wrote.
fn unpack(zip_path: &Path, into: &Path, keep: fn(&Path) -> bool, flat: bool) -> Result<usize, String> {
    let file = std::fs::File::open(zip_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("The download isn't a zip file: {e}"))?;
    let mut presets = 0;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        if entry.is_dir() {
            continue;
        }
        // `enclosed_name` refuses paths that would leave the folder (`..`, absolute).
        let Some(name) = entry.enclosed_name() else { continue };
        let rel: PathBuf = name.components().skip(1).collect();
        if rel.as_os_str().is_empty() || !keep(&rel) {
            continue;
        }
        let rel = if flat { PathBuf::from(rel.file_name().unwrap_or_default()) } else { rel };
        let out = into.join(&rel);
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("Couldn't create {}: {e}", parent.display()))?;
        }
        let mut f = std::fs::File::create(&out).map_err(|e| format!("Couldn't write {}: {e}", out.display()))?;
        std::io::copy(&mut entry, &mut f).map_err(|e| e.to_string())?;
        if ext(&rel) == "milk" {
            presets += 1;
        }
    }
    Ok(presets)
}

/// One line for the dialog and the status bar.
pub fn describe(p: &Progress) -> String {
    let mb = |b: u64| b as f64 / 1_048_576.0;
    match p {
        Progress::Idle => String::new(),
        Progress::Running { step, done, total: Some(t) } if *t > 0 => {
            format!("{step}… {:.0} of {:.0} MB", mb(*done), mb(*t))
        }
        Progress::Running { step, done, .. } if *done > 0 => format!("{step}… {:.1} MB", mb(*done)),
        Progress::Running { step, .. } => format!("{step}…"),
        Progress::Done { presets } => format!("Added {presets} MilkDrop presets to the MilkDrop folder."),
        Progress::Failed(e) => format!("Couldn't add the community pack: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unpacks_presets_and_flattens_textures_safely() {
        let dir = std::env::temp_dir().join(format!("openvj-pack-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let zip_path = dir.join("p.zip");
        {
            let mut z = zip::ZipWriter::new(std::fs::File::create(&zip_path).unwrap());
            let opts = zip::write::SimpleFileOptions::default();
            for (name, body) in [
                ("pack-master/LICENSE.md", "licence"),
                ("pack-master/Dancer/Glow/a.milk", "[preset00]"),
                ("pack-master/Fractal/b.MILK", "[preset00]"),
                ("pack-master/Fractal/notes.txt", "skip"),
                ("pack-master/textures/clouds.jpg", "jpg"),
                ("pack-master/../../evil.milk", "no"),
            ] {
                z.start_file(name, opts).unwrap();
                z.write_all(body.as_bytes()).unwrap();
            }
            z.finish().unwrap();
        }
        let out = dir.join("out");
        assert_eq!(unpack(&zip_path, &out, is_pack_file, false).unwrap(), 2);
        assert!(out.join("Dancer/Glow/a.milk").is_file());
        assert!(out.join("Fractal/b.MILK").is_file());
        assert!(out.join("LICENSE.md").is_file());
        assert!(!out.join("Fractal/notes.txt").exists());
        assert!(!dir.join("evil.milk").exists() && !std::env::temp_dir().join("evil.milk").exists());

        let tex = dir.join("textures");
        assert_eq!(unpack(&zip_path, &tex, is_texture, true).unwrap(), 0);
        assert!(tex.join("clouds.jpg").is_file());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn describes_progress() {
        let p = Progress::Running { step: "Downloading presets".into(), done: 5 << 20, total: Some(20 << 20) };
        assert_eq!(describe(&p), "Downloading presets… 5 of 20 MB");
        assert_eq!(describe(&Progress::Done { presets: 3 }), "Added 3 MilkDrop presets to the MilkDrop folder.");
    }
}
