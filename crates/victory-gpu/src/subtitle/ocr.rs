//! Live English subtitle strip capture from the swap chain's back buffer and
//! asynchronous WinRT OCR (`Windows.Media.Ocr`) on a low-priority background
//! thread. Zero GPU stalls, zero per-frame allocations: a 2-slot ring of tiny
//! (~1 MiB) `READBACK` buffers copies only the bottom-center subtitle strip
//! every ~200 ms right before our Turkish box is drawn over it.

use std::sync::{Condvar, Mutex, OnceLock};
use std::time::{Duration, Instant};

use windows::Win32::Graphics::Direct3D12::*;
use windows::Win32::Graphics::Dxgi::Common::*;

use crate::host::{Host, HostList, transition};

/// Default interval between subtitle strip captures (5 Hz).
const DEFAULT_INTERVAL_MS: u64 = 200;

/// Number of persistent readback ring slots (covers 2-buffer swap chain + late present).
const SLOTS: usize = 2;

/// Minimum bright-pixel fraction for a strip to possibly hold subtitles.
const MIN_TEXT_PROMILLE: usize = 1; // 0.1%
/// Above this fraction the strip is a bright flash / white screen, not subtitles.
const MAX_TEXT_PROMILLE: usize = 380; // 38%

/// How many consecutive empty OCR frames are needed before clearing the active
/// subtitle line (eliminates single-frame flicker on camera cuts).
const EMPTY_DEBOUNCE: u32 = 2;

fn sample_interval() -> Duration {
    static INTERVAL: OnceLock<Duration> = OnceLock::new();
    *INTERVAL.get_or_init(|| {
        let ms = std::env::var("VICTORY_SUBS_INTERVAL_MS")
            .ok()
            .and_then(|v| v.trim().parse::<u64>().ok())
            .map_or(DEFAULT_INTERVAL_MS, |v| v.clamp(100, 2_000));
        Duration::from_millis(ms)
    })
}

/// Optional test subtitle (`VICTORY_SUBS_TEST=<english text>`) for instant
/// overlay preview even on title screens.
fn test_override() -> Option<&'static str> {
    static TEST: OnceLock<Option<String>> = OnceLock::new();
    TEST.get_or_init(|| {
        std::env::var("VICTORY_SUBS_TEST")
            .ok()
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty())
    })
    .as_deref()
}

/// Supported swap-chain formats and their bytes-per-pixel.
fn format_bpp(format: DXGI_FORMAT) -> Option<u32> {
    match format {
        DXGI_FORMAT_R8G8B8A8_UNORM | DXGI_FORMAT_B8G8R8A8_UNORM | DXGI_FORMAT_R10G10B10A2_UNORM => Some(4),
        DXGI_FORMAT_R16G16B16A16_FLOAT => Some(8),
        _ => None,
    }
}

/// Bottom-center ROI rectangle `(x, y, w, h)` covering GTA's native English
/// subtitle band (middle 76% width, bottom 77%..96% height).
pub(crate) fn roi_for(width: u32, height: u32) -> (u32, u32, u32, u32) {
    let x = (width * 12 / 100) & !3;
    let w = ((width * 76 / 100) & !3).max(4).min(width.saturating_sub(x));
    let y = (height * 77 / 100) & !1;
    let h = ((height * 19 / 100) & !1).max(4).min(height.saturating_sub(y));
    (x, y, w, h)
}

struct ReadbackSlot {
    buffer: ID3D12Resource,
    footprint: D3D12_PLACED_SUBRESOURCE_FOOTPRINT,
    total_bytes: usize,
    roi: (u32, u32, u32, u32),
    format: DXGI_FORMAT,
    in_flight: bool,
    /// Submission fence number to wait for (`0` = synchronous execute, ready immediately).
    flown: u64,
}

// SAFETY: D3D12 resources are free-threaded; slots are mutated only under `Presenter`'s lock.
unsafe impl Send for ReadbackSlot {}

impl ReadbackSlot {
    fn new(host: &Host, width: u32, height: u32, format: DXGI_FORMAT) -> Option<Self> {
        let bpp = format_bpp(format)?;
        let roi = roi_for(width, height);
        if roi.2 < 16 || roi.3 < 16 {
            return None;
        }
        let pitch = (roi.2 * bpp).next_multiple_of(256);
        let total_bytes = pitch as usize * roi.3 as usize;
        let heap = D3D12_HEAP_PROPERTIES {
            Type: D3D12_HEAP_TYPE_READBACK,
            ..Default::default()
        };
        let desc = D3D12_RESOURCE_DESC {
            Dimension: D3D12_RESOURCE_DIMENSION_BUFFER,
            Width: total_bytes as u64,
            Height: 1,
            DepthOrArraySize: 1,
            MipLevels: 1,
            SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
            Layout: D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
            ..Default::default()
        };
        let mut buffer: Option<ID3D12Resource> = None;
        // SAFETY: plain readback buffer creation.
        unsafe {
            host.device
                .CreateCommittedResource(
                    &heap,
                    D3D12_HEAP_FLAG_NONE,
                    &desc,
                    D3D12_RESOURCE_STATE_COPY_DEST,
                    None,
                    &mut buffer,
                )
                .ok()?;
        }
        let footprint = D3D12_PLACED_SUBRESOURCE_FOOTPRINT {
            Offset: 0,
            Footprint: D3D12_SUBRESOURCE_FOOTPRINT {
                Format: format,
                Width: roi.2,
                Height: roi.3,
                Depth: 1,
                RowPitch: pitch,
            },
        };
        Some(Self {
            buffer: buffer?,
            footprint,
            total_bytes,
            roi,
            format,
            in_flight: false,
            flown: 0,
        })
    }
}

/// GPU strip capturer owned by [`super::overlay::Subtitles`].
pub(super) struct Capturer {
    slots: [Option<ReadbackSlot>; SLOTS],
    last_sample: Instant,
    last_key: (u32, u32, i32),
    failed: bool,
}

impl Capturer {
    pub(super) fn new() -> Self {
        Self {
            slots: [None, None],
            last_sample: Instant::now() - Duration::from_secs(10),
            last_key: (0, 0, 0),
            failed: false,
        }
    }

    /// Polls completed readback slots (never waits!) and, if the sample
    /// interval has elapsed, records a tiny ROI copy from `back` (which must
    /// be in `COPY_DEST` and is returned to `COPY_DEST`).
    pub(super) fn record(
        &mut self,
        host: &Host,
        copy: &HostList,
        back: &ID3D12Resource,
        (width, height): (u32, u32),
        format: DXGI_FORMAT,
    ) -> Option<usize> {
        if self.failed || test_override().is_some() || format_bpp(format).is_none() {
            return None;
        }
        if width < 320 || height < 240 {
            return None;
        }

        let finished = crate::host::submissions().1;
        // 1. Harvest any finished readback slot without blocking.
        for slot_opt in &mut self.slots {
            if let Some(slot) = slot_opt
                && slot.in_flight
                && (slot.flown == 0 || finished >= slot.flown)
            {
                harvest_slot(slot);
                slot.in_flight = false;
            }
        }

        // If swap chain size/format changed, retire old slots and recreate lazily.
        let key = (width, height, format.0);
        if self.last_key != key {
            for slot_opt in &mut self.slots {
                if let Some(old) = slot_opt.take() {
                    crate::host::retire_resource(old.buffer);
                }
            }
            self.last_key = key;
        }

        if self.last_sample.elapsed() < sample_interval() {
            return None;
        }

        // 2. Pick a free slot to record the next ROI copy.
        let free_idx = self.slots.iter().position(|s| s.as_ref().is_none_or(|r| !r.in_flight))?;
        if self.slots[free_idx].is_none() {
            let Some(made) = ReadbackSlot::new(host, width, height, format) else {
                self.failed = true;
                return None;
            };
            self.slots[free_idx] = Some(made);
        }
        let slot = self.slots[free_idx].as_mut()?;
        let (rx, ry, rw, rh) = slot.roi;
        copy.keep(&slot.buffer);
        let list = &copy.list;
        // SAFETY: `list` is open, `back` is in `COPY_DEST` and lives through
        // submission, `slot.buffer` is kept alive and large enough for `roi`.
        unsafe {
            list.ResourceBarrier(&[transition(
                back,
                D3D12_RESOURCE_STATE_COPY_DEST,
                D3D12_RESOURCE_STATE_COPY_SOURCE,
            )]);
            let to = D3D12_TEXTURE_COPY_LOCATION {
                pResource: std::mem::transmute_copy(&slot.buffer),
                Type: D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT,
                Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 { PlacedFootprint: slot.footprint },
            };
            let from = D3D12_TEXTURE_COPY_LOCATION {
                pResource: std::mem::transmute_copy(back),
                Type: D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
                Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 { SubresourceIndex: 0 },
            };
            let src_box = D3D12_BOX {
                left: rx,
                top: ry,
                front: 0,
                right: rx + rw,
                bottom: ry + rh,
                back: 1,
            };
            list.CopyTextureRegion(&to, 0, 0, 0, &from, Some(&src_box));
            list.ResourceBarrier(&[transition(
                back,
                D3D12_RESOURCE_STATE_COPY_SOURCE,
                D3D12_RESOURCE_STATE_COPY_DEST,
            )]);
        }
        self.last_sample = Instant::now();
        Some(free_idx)
    }

    /// Tags the recorded readback slot with the submission fence `flown`.
    pub(super) fn submitted(&mut self, slot: Option<usize>, flown: u64) {
        let Some(idx) = slot else { return };
        if let Some(Some(s)) = self.slots.get_mut(idx) {
            s.flown = flown;
            s.in_flight = true;
        }
    }
}

/// Raw ROI pixels handed to the background OCR worker.
struct RawStrip {
    width: u32,
    height: u32,
    pitch: usize,
    format: DXGI_FORMAT,
    bytes: Vec<u8>,
}

fn harvest_slot(slot: &ReadbackSlot) {
    let mut ptr = std::ptr::null_mut();
    let read_range = D3D12_RANGE { Begin: 0, End: slot.total_bytes };
    let write_none = D3D12_RANGE { Begin: 0, End: 0 };
    // SAFETY: the GPU fence for this slot has completed; `total_bytes` fits the buffer.
    unsafe {
        if slot.buffer.Map(0, Some(&read_range), Some(&mut ptr)).is_err() || ptr.is_null() {
            return;
        }
        let slice = std::slice::from_raw_parts(ptr.cast::<u8>(), slot.total_bytes);
        let strip = RawStrip {
            width: slot.roi.2,
            height: slot.roi.3,
            pitch: slot.footprint.Footprint.RowPitch as usize,
            format: slot.format,
            bytes: slice.to_vec(),
        };
        slot.buffer.Unmap(0, Some(&write_none));
        push_strip(strip);
    }
}

struct OcrWorker {
    pending: Mutex<Option<RawStrip>>,
    wake: Condvar,
    latest: Mutex<Option<String>>,
}

fn ocr_worker() -> &'static OcrWorker {
    static WORKER: OnceLock<&'static OcrWorker> = OnceLock::new();
    WORKER.get_or_init(|| {
        let state: &'static OcrWorker = Box::leak(Box::new(OcrWorker {
            pending: Mutex::new(None),
            wake: Condvar::new(),
            latest: Mutex::new(None),
        }));
        let _ = std::thread::Builder::new()
            .name("victory-subs-ocr".into())
            .spawn(move || run_ocr_worker(state));
        state
    })
}

fn push_strip(strip: RawStrip) {
    let w = ocr_worker();
    *w.pending.lock().unwrap_or_else(|e| e.into_inner()) = Some(strip);
    w.wake.notify_one();
}

/// Latest recognized English subtitle line from OCR (or `VICTORY_SUBS_TEST`).
pub(crate) fn current_en() -> Option<String> {
    if let Some(test) = test_override() {
        return Some(test.to_owned());
    }
    let w = ocr_worker();
    w.latest.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

fn run_ocr_worker(state: &'static OcrWorker) {
    use windows::Globalization::Language;
    use windows::Graphics::Imaging::{BitmapPixelFormat, SoftwareBitmap};
    use windows::Media::Ocr::OcrEngine;
    use windows::Storage::Streams::DataWriter;
    use windows::Win32::System::Threading::{
        GetCurrentThread, SetThreadPriority, THREAD_PRIORITY_BELOW_NORMAL,
    };
    use windows::Win32::System::WinRT::{RO_INIT_MULTITHREADED, RoInitialize, RoUninitialize};
    use windows::core::HSTRING;

    // SAFETY: lowering our own background thread priority and initializing WinRT MTA.
    let ro_inited = unsafe {
        let _ = SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_BELOW_NORMAL);
        RoInitialize(RO_INIT_MULTITHREADED).is_ok()
    };

    let engine = ["en-US", "en"]
        .iter()
        .find_map(|tag| {
            Language::CreateLanguage(&HSTRING::from(*tag))
                .ok()
                .and_then(|lang| OcrEngine::TryCreateFromLanguage(&lang).ok())
        })
        .or_else(|| OcrEngine::TryCreateFromUserProfileLanguages().ok());

    let Some(engine) = engine else {
        tracing::warn!("subtitles: Windows OCR engine unavailable (install English OCR pack for live capture)");
        if ro_inited {
            // SAFETY: balanced with `RoInitialize` above.
            unsafe { RoUninitialize() };
        }
        return;
    };
    tracing::info!("subtitles: live WinRT OCR worker ready (F8 to toggle, tr_subtitles_cache.json)");

    let mut last_hash = 0u64;
    let mut empty_streak = 0u32;

    loop {
        let strip = {
            let mut guard = state.pending.lock().unwrap_or_else(|e| e.into_inner());
            loop {
                if let Some(s) = guard.take() {
                    break s;
                }
                guard = state.wake.wait(guard).unwrap_or_else(|e| e.into_inner());
            }
        };

        let Some((bgra, mask_hash)) = preprocess_strip(&strip) else {
            empty_streak = empty_streak.saturating_add(1);
            if empty_streak >= EMPTY_DEBOUNCE {
                let mut latest = state.latest.lock().unwrap_or_else(|e| e.into_inner());
                if latest.is_some() {
                    *latest = None;
                }
                last_hash = 0;
            }
            continue;
        };

        if mask_hash == last_hash {
            empty_streak = 0;
            continue;
        }
        last_hash = mask_hash;

        let recognized = (|| -> windows::core::Result<String> {
            let writer = DataWriter::new()?;
            writer.WriteBytes(&bgra)?;
            let buf = writer.DetachBuffer()?;
            let bmp = SoftwareBitmap::CreateCopyFromBuffer(
                &buf,
                BitmapPixelFormat::Bgra8,
                strip.width as i32,
                strip.height as i32,
            )?;
            let result = engine.RecognizeAsync(&bmp)?.join()?;
            let mut lines_out = Vec::new();
            if let Ok(lines) = result.Lines() {
                for line in lines {
                    if let Ok(t) = line.Text() {
                        let s = t.to_string();
                        if !s.trim().is_empty() {
                            lines_out.push(s);
                        }
                    }
                }
            }
            if lines_out.is_empty() {
                Ok(result.Text()?.to_string())
            } else {
                Ok(lines_out.join(" "))
            }
        })();

        match recognized.ok().and_then(|raw| clean_ocr_text(&raw)) {
            Some(clean) => {
                empty_streak = 0;
                *state.latest.lock().unwrap_or_else(|e| e.into_inner()) = Some(clean);
            }
            None => {
                empty_streak = empty_streak.saturating_add(1);
                if empty_streak >= EMPTY_DEBOUNCE {
                    *state.latest.lock().unwrap_or_else(|e| e.into_inner()) = None;
                }
            }
        }
    }
}

/// Converts a captured DXGI strip into a crisp high-contrast black-on-white
/// BGRA8 image isolating GTA-style bright white/warm-yellow subtitles, plus a
/// coarse change hash. Returns `None` when the strip holds no subtitle text.
fn preprocess_strip(strip: &RawStrip) -> Option<(Vec<u8>, u64)> {
    let w = strip.width as usize;
    let h = strip.height as usize;
    if w == 0 || h == 0 {
        return None;
    }
    let mut bgra = vec![255u8; w * h * 4];
    let mut ink_count = 0usize;
    let mut hash = 0xcbf2_9ce4_8422_2325u64;

    for y in 0..h {
        let row_start = y * strip.pitch;
        if row_start >= strip.bytes.len() {
            break;
        }
        let row = &strip.bytes[row_start..];
        let out_row = &mut bgra[y * w * 4..(y + 1) * w * 4];
        for x in 0..w {
            let [r, g, b] = decode_rgb(strip.format, row, x);
            if is_subtitle_ink(r, g, b) {
                ink_count += 1;
                let dst = x * 4;
                out_row[dst] = 0;
                out_row[dst + 1] = 0;
                out_row[dst + 2] = 0;
                out_row[dst + 3] = 255;
                if (x & 3) == 0 && (y & 1) == 0 {
                    hash ^= ((y as u64) << 16) | (x as u64);
                    hash = hash.wrapping_mul(0x1000_0000_01b3);
                }
            }
        }
    }

    let total = (w * h).max(1);
    let promille = ink_count * 1000 / total;
    if !(MIN_TEXT_PROMILLE..=MAX_TEXT_PROMILLE).contains(&promille) {
        return None;
    }
    Some((bgra, hash))
}

#[inline]
fn is_subtitle_ink(r: u8, g: u8, b: u8) -> bool {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    // Pure / off-white subtitle glyph pixels.
    let white_text = min >= 185 && (max - min) <= 55;
    // Warm-yellow highlighted subtitle / speaker words.
    let yellow_text = r >= 205 && g >= 185 && b <= 135;
    white_text || yellow_text
}

#[inline]
fn decode_rgb(format: DXGI_FORMAT, row: &[u8], x: usize) -> [u8; 3] {
    match format {
        DXGI_FORMAT_R8G8B8A8_UNORM => {
            let at = x * 4;
            if at + 3 <= row.len() { [row[at], row[at + 1], row[at + 2]] } else { [0; 3] }
        }
        DXGI_FORMAT_B8G8R8A8_UNORM => {
            let at = x * 4;
            if at + 3 <= row.len() { [row[at + 2], row[at + 1], row[at]] } else { [0; 3] }
        }
        DXGI_FORMAT_R10G10B10A2_UNORM => {
            let at = x * 4;
            if at + 4 <= row.len() {
                let w = u32::from_le_bytes([row[at], row[at + 1], row[at + 2], row[at + 3]]);
                [
                    ((w & 0x3FF) * 255 / 1023) as u8,
                    (((w >> 10) & 0x3FF) * 255 / 1023) as u8,
                    (((w >> 20) & 0x3FF) * 255 / 1023) as u8,
                ]
            } else {
                [0; 3]
            }
        }
        DXGI_FORMAT_R16G16B16A16_FLOAT => {
            let at = x * 8;
            if at + 6 <= row.len() {
                let to_u8 = |lo: u8, hi: u8| {
                    let h = u16::from_le_bytes([lo, hi]);
                    let v = f16_to_f32(h).clamp(0.0, 1.0);
                    (v.sqrt() * 255.0).round() as u8
                };
                [
                    to_u8(row[at], row[at + 1]),
                    to_u8(row[at + 2], row[at + 3]),
                    to_u8(row[at + 4], row[at + 5]),
                ]
            } else {
                [0; 3]
            }
        }
        _ => [0; 3],
    }
}

#[inline]
fn f16_to_f32(h: u16) -> f32 {
    let sign = (h >> 15) & 1;
    let exp = ((h >> 10) & 0x1F) as i32;
    let mant = (h & 0x3FF) as f32;
    let v = match exp {
        0 => mant * 2f32.powi(-24),
        31 => 1.0,
        e => (1.0 + mant / 1024.0) * 2f32.powi(e - 15),
    };
    if sign != 0 { -v } else { v }
}

/// Normalizes raw OCR text and filters HUD noise / stray artifacts.
pub(crate) fn clean_ocr_text(raw: &str) -> Option<String> {
    let mut words: Vec<String> = Vec::new();
    for tok in raw.split_whitespace() {
        let fixed = if tok == "|" { "I" } else { tok };
        let trimmed = fixed.trim_matches(|c: char| matches!(c, '_' | '~' | '^' | '`' | '|' | '•'));
        if !trimmed.is_empty() {
            words.push(trimmed.to_owned());
        }
    }
    if words.is_empty() {
        return None;
    }
    let joined = words.join(" ");
    let letters = joined.chars().filter(|c| c.is_ascii_alphabetic()).count();
    let total_non_space = joined.chars().filter(|c| !c.is_whitespace()).count();
    // Reject stray 1-2 character noise or mostly numeric HUD counters.
    if letters < 3 || letters * 2 < total_non_space {
        return None;
    }
    Some(joined)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roi_fits_inside_1080p_and_720p() {
        for (w, h) in [(1920, 1080), (1280, 720), (2560, 1440), (3840, 2160)] {
            let (rx, ry, rw, rh) = roi_for(w, h);
            assert!(rx + rw <= w, "{w}x{h}");
            assert!(ry + rh <= h, "{w}x{h}");
            assert!(rw >= w / 2 && rh >= h / 10);
        }
    }

    #[test]
    fn ocr_cleaner_rejects_noise_and_keeps_dialogue() {
        assert_eq!(clean_ocr_text("  |  "), None);
        assert_eq!(clean_ocr_text("30 FPS 12:45"), None);
        assert_eq!(
            clean_ocr_text("  |  don't   know what the fuck happened. "),
            Some("I don't know what the fuck happened.".to_owned())
        );
    }

    #[test]
    fn preprocessor_detects_white_subtitles_and_ignores_black() {
        let black = RawStrip {
            width: 100,
            height: 20,
            pitch: 400,
            format: DXGI_FORMAT_R8G8B8A8_UNORM,
            bytes: vec![0u8; 400 * 20],
        };
        assert!(preprocess_strip(&black).is_none());

        let mut with_text = black;
        for x in 10..60 {
            let at = 10 * 400 + x * 4;
            with_text.bytes[at..at + 4].copy_from_slice(&[250, 250, 250, 255]);
        }
        let (bgra, hash) = preprocess_strip(&with_text).expect("detects bright text");
        assert_ne!(hash, 0);
        // White subtitle pixel becomes crisp black ink on white background.
        let sample = 10 * 100 * 4 + 20 * 4;
        assert_eq!(&bgra[sample..sample + 4], &[0, 0, 0, 255]);
        assert_eq!(&bgra[0..4], &[255, 255, 255, 255]);
    }
}
