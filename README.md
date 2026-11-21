# Syntetika OpenVJ

Syntetika OpenVJ adalah aplikasi VJ open-source berbasis Rust untuk mixing clip, generator GPU, efek visual, dan output fullscreen. Proyek ini menggunakan fondasi [openVJ](https://github.com/openwave-studio/openvj) dan ditujukan untuk pengembangan Windows terlebih dahulu.

Syntetika bukan salinan kode, aset, atau identitas proprietary Resolume. Dukungan import `.avc` hanya ditujukan untuk interoperabilitas dengan format yang dapat dibaca secara independen.

## Status proyek

- Versi pengembangan: `0.1.0`
- Target utama saat ini: **Windows 10/11 64-bit**
- Backend GPU Windows: **DirectX 12**
- GPU target awal: Intel UHD integrated graphics
- Lisensi: **GPL-3.0-or-later**
- Belum tersedia installer Windows resmi; distribusi saat ini berupa portable ZIP

## Fitur yang tersedia

- Clip grid dengan layer dan column
- Deck untuk beberapa set clip
- Generator GPU: Plasma, Tunnel, Bars, Rings, Noise, Kaleido, Gradient, Strobe, Checker
- Gambar dan video melalui FFmpeg
- Decode GPU untuk DXV dan HAP jika GPU mendukung BC textures
- Kamera dan capture device melalui fitur `camera`
- Efek ISF dan shader generator
- Efek clip, layer, group, dan composition
- Blend mode, opacity, bypass, solo, transform, crossfade, dan feedback
- Playback Loop, Bounce, Random, Play Once, Toggle, dan Piano
- Beat sync, BPM, audio FFT, audio-reactive mapping, serta MIDI input
- Piano Roll MIDI per clip: velocity lane, marquee select, geser banyak note, undo/redo, copy/paste/duplicate, playhead, fold, scale assist, minimap, mode Draw/Select/Erase, zoom, dan quantize
- Import komposisi Resolume `.avc` secara parsial
- Output fullscreen dan dasar projection mapping
- Preview dan output monitor
- Performance telemetry: FPS, render time, render passes, dan dropped frames
- Performance Mode:
  - **Eco:** 30 FPS, preview 15 FPS
  - **Balanced:** 45 FPS, preview 30 FPS
  - **Quality:** 60 FPS, preview 60 FPS

## Build Windows

### Prasyarat

- Rust stable dan Cargo
- Visual Studio Build Tools dengan workload **Desktop development with C++**
- Windows SDK
- FFmpeg untuk video non-DXV/HAP
- Driver Intel/NVIDIA/AMD terbaru

### Build

```powershell
git clone https://github.com/danartriyudistira/syntetika-openvj.git
cd syntetika-openvj
cargo build --release -p openvj-app --no-default-features --features camera
```

Executable hasil build:

```text
target/release/syntetika-openvj.exe
```

Jalankan:

```powershell
.\target\release\syntetika-openvj.exe
```

Untuk menjalankan pada FPS tertentu:

```powershell
$env:SYNTHETIKA_FPS=45
.\target\release\syntetika-openvj.exe
```

Nilai yang didukung: `30`, `45`, dan `60`.

## FFmpeg

Release Windows portable menyertakan `ffmpeg.exe` dan `ffprobe.exe` di folder yang sama dengan aplikasi. Syntetika menjalankan FFmpeg sebagai proses decoder terpisah, sehingga executable utama tetap GPL-3.0-or-later dan binary FFmpeg mengikuti LGPL-nya sendiri.

Build dari source atau pengguna yang memakai paket tanpa FFmpeg dapat mengatur:

```powershell
$env:OPENVJ_FFMPEG_DIR="C:\path\to\ffmpeg\bin"
```

DXV dan HAP menggunakan decoder internal jika GPU mendukung block-compressed textures. Lisensi FFmpeg Windows tersedia di `FFmpeg-LICENSE.txt` pada paket release.

## Contoh efek ISF

Efek ISF dapat disimpan sebagai file `.fs` di folder Effects. Contoh minimal:

```glsl
/*{
    "DESCRIPTION": "Simple tint effect",
    "INPUTS": [
        {"NAME": "inputImage", "TYPE": "image"},
        {"NAME": "amount", "TYPE": "float", "MIN": 0.0, "MAX": 1.0, "DEFAULT": 0.5}
    ]
}*/
void main() {
    vec4 color = IMG_THIS_PIXEL(inputImage);
    gl_FragColor = vec4(mix(color.rgb, color.rgb * vec3(0.7, 1.0, 0.7), amount), color.a);
}
```

## Kontrol dasar

| Kontrol | Fungsi |
|---|---|
| Klik clip | Launch clip |
| Klik header column | Launch satu column |
| Klik kanan clip | Pilih tanpa launch |
| `Ctrl+1` sampai `Ctrl+4` | Ganti layout |
| `Tab` | Buka library |
| `Delete` | Hapus clip terpilih |
| `Ctrl+M` | MIDI learn |
| `F` pada output window | Fullscreen |

### Piano Roll

Tab **Piano Roll** aktif untuk clip terpilih. Shortcut berikut hanya berlaku saat tab ini terbuka:

| Kontrol | Fungsi |
|---|---|
| Mode `Draw` / `Select` / `Erase` | Klik grid menambah note / drag marquee / klik note menghapus |
| Drag di grid kosong | Marquee selection |
| `Shift` + klik note | Tambah atau lepas note dari selection |
| Drag note | Geser seluruh note terpilih |
| Drag di velocity lane | Ubah velocity note terdekat |
| `Ctrl+A` / `Esc` | Pilih semua / lepas selection |
| `Delete` | Hapus note terpilih |
| `Ctrl+C` / `Ctrl+V` / `Ctrl+D` | Copy / paste di playhead / duplicate |
| `Ctrl+Z` / `Ctrl+Shift+Z` / `Ctrl+Y` | Undo / redo |
| Tombol `Fold` | Tampilkan hanya pitch yang dipakai clip |
| Picker `Key` + `Scale` | Sorot baris yang masuk skala; baris di luar skala diredupkan |
| `Zoom −` / `Zoom +` | Skala horizontal grid (50%–400%) |
| `Rows −` / `Rows +` | Tinggi baris pitch |
| `Expand` / `Shrink` | Perbesar panel editor |
| `Quantize` | Snap semua note terpilih ke grid |

- Gutters di kiri menampilkan nama nada (`C4`, `F#3`), atau nama drum GM (`Kick`, `Snare`, `CH`) pada channel 10.
- Semua penempatan note di-snap ke Grid yang dipilih.
- Minimap di bawah grid menampilkan seluruh roll, posisi playhead, dan area yang sedang terlihat.

Paste dan duplicate memakai posisi playhead clip, dibulatkan ke grid 1/16, dan note yang melewati batas bar dilewati.

## Batasan saat ini

- Audio track dari video belum diputar.
- Image sequence belum tersedia.
- Video selain DXV/HAP didecode di CPU melalui FFmpeg.
- Ableton Link tidak aktif pada build Windows default karena build menggunakan `--no-default-features`.
- Syphon hanya tersedia pada macOS.
- Spout belum tersedia pada Windows.
- NDI input/output belum tersedia.
- MIDI LED feedback belum tersedia.
- Piano Roll belum memiliki quantize otomatis dan time-stretch.
- Export MIDI preset ke Resolume belum tersedia.
- Mapping parameter clip, clip effect, group control, Synthesis knob, dan macro belum lengkap.
- Parameter animation/envelope/dashboard belum selesai.
- Import `.avc` dapat melewati Wire patch, text generator, NDI, router tertentu, effect yang tidak memiliki padanan, dan sebagian transform 3D.
- Linear dan Bezier grid warp dari preset Resolume dapat diimport tetapi belum seluruhnya dapat diedit.
- Belum ada installer Windows; distribusi saat ini berupa portable ZIP.

## Pengujian

Test yang relevan dapat dijalankan per crate:

```powershell
cargo test -p openvj-core --lib
cargo test -p openvj-audio --lib --no-default-features
cargo test -p openvj-media --lib --no-default-features
cargo test -p openvj-resolume --lib --no-default-features
```

Pada Windows, test GPU renderer sebaiknya menggunakan DirectX 12:

```powershell
$env:WGPU_BACKEND="dx12"
cargo test -p openvj-render --lib --no-default-features
```

## Lisensi dan atribusi

Syntetika OpenVJ menggunakan GPL-3.0-or-later. Atribusi dan lisensi pihak ketiga tercantum dalam [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

Syntetika mempertahankan atribusi openVJ serta tidak mendistribusikan kode atau aset proprietary Resolume.
