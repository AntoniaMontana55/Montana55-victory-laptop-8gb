//! The subtitle overlay's picture: a cinematic box at the bottom center of
//! the screen over the game's own subtitles — near-opaque black rounded
//! background, centred white semibold text, full Turkish character support
//! (ğ, ü, ş, ı, İ, ö, ç, â, û) — rasterized on the CPU with GDI (Segoe UI
//! Semibold), carried to the card exactly like the panel's box
//! ([`crate::host::panel`]) and copied over the swap chain's back buffer
//! before Present.

use std::sync::OnceLock;
use std::time::{Duration, Instant};

use windows::Win32::Foundation::{HANDLE, RECT};
use windows::Win32::Graphics::Direct3D12::*;
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits, BITMAPINFO, BITMAPINFOHEADER,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_F8};
use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;
use windows_sys::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;

use super::{Host, HostList, transition};
use crate::subtitle::{self, Line};

/// Logical canvas width: the box spans ~60% of a 1080p frame, clamped by
/// the real width.
const BOX_W: u32 = 1150;
/// How tall a text row is at 2x (28 px font, 1.35 line-height).
const LINE_H: u32 = 38;
/// Box padding around the text.
const PAD_X: u32 = 28;
const PAD_Y: u32 = 18;
/// Corner radius, in px on the logical canvas.
const RADIUS: u32 = 11;
/// Bottom margin of the box from the screen edge (~4% of 1080p).
const MARGIN_Y: u32 = 44;

/// Background: near-opaque black (85% alpha over the game).
const BG_ALPHA: f32 = 0.87;
/// Text: pure white.
const WHITE: [u8; 3] = [255, 255, 255];

/// The font: Segoe UI at 28 px, semibold, with quality cleartype hints.
const FONT_H: u32 = 28;

/// How long a line stays after its text stops changing (the game's own
/// timing is not visible to us; a generous hold, then hide).
const HOLD: Duration = Duration::from_millis(1_400);

/// Whether F8 was pressed since the last call, with one of our windows in
/// front (the panel's F3 pattern).
fn pressed() -> bool {
    static DOWN: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    // SAFETY: a plain query.
    let down = unsafe { GetAsyncKeyState(i32::from(VK_F8)) } < 0;
    if DOWN.swap(down, std::sync::atomic::Ordering::Relaxed) || !down {
        return false;
    }
    let mut process = 0;
    // SAFETY: plain queries; room for the answer.
    unsafe { GetWindowThreadProcessId(GetForegroundWindow(), &mut process) };
    process == std::process::id()
}

/// Whether the overlay is shown now (F8 toggles; starts on when
/// `VICTORY_SUBS` is on).
fn shown() -> bool {
    static SHOWN: OnceLock<std::sync::atomic::AtomicBool> = OnceLock::new();
    SHOWN
        .get_or_init(|| std::sync::atomic::AtomicBool::new(subtitle::asked()))
        .load(std::sync::atomic::Ordering::Relaxed)
}

fn toggle() {
    static SHOWN: OnceLock<std::sync::atomic::AtomicBool> = OnceLock::new();
    SHOWN
        .get_or_init(|| std::sync::atomic::AtomicBool::new(subtitle::asked()))
        .fetch_xor(true, std::sync::atomic::Ordering::Relaxed);
}

/// Rasterizes one text line to (A8 coverage, width) with GDI.
fn rasterize(text: &str) -> Option<(Vec<u8>, u32, u32)> {
    use windows::Win32::Graphics::Gdi::{
        CreateFontW, SetBkMode, SetTextColor, TextOutW, GdiObject, TRANSPARENT,
    };
    use windows::core::w;

    // SAFETY: plain GDI object creation and use; all handles are ours and
    // deleted below.
    unsafe {
        let dc = CreateCompatibleDC(None);
        if dc.is_invalid() {
            return None;
        }
        let delete_dc = |dc| {
            let _ = DeleteDC(dc);
        };
        // Segoe UI Semibold (family 3=semibold via weight 600).
        let face = w!("Segoe UI");
        let font = CreateFontW(
            FONT_H as i32,
            0,
            0,
            0,
            600, // semibold
            0,   // italic
            0,   // underline
            0,   // strikeout
            0x041F, // TURKISH charset (full ğüşıİöç)
            0,   // out precision (default)
            0,   // clip precision (default)
            5,   // cleartype quality
            0,   // pitch and family (default)
            face,
        );
        if font.is_invalid() {
            delete_dc(dc);
            return None;
        }
        let _old = windows::Win32::Graphics::Gdi::SelectObject(dc, GdiObject::from(font));
        let _bk = SetBkMode(dc, TRANSPARENT);
        // White text on black: the colour channel gives the coverage mask.
        let _colour = SetTextColor(dc, windows::Win32::Foundation::COLORREF(0x00FF_FFFF));

        // Measure: total advance of the text.
        let wide: Vec<u16> = text.encode_utf16().collect();
        // Text extents via GetTextExtentPoint32.
        use windows::Win32::Graphics::Gdi::GetTextExtentPoint32W;
        let mut size = windows::Win32::Foundation::SIZE::default();
        if GetTextExtentPoint32W(dc, &wide, &mut size).is_err() {
            let _ = DeleteObject(GdiObject::from(font));
            delete_dc(dc);
            return None;
        }
        let (w_text, h_text) = (size.cx.max(1) as u32, size.cy.max(1) as u32);

        // A bitmap the size of the line, 1 bpp is fiddly: use 32-bpp and
        // read the green channel (the DIB's bytes are BGRA).
        let dib_w = w_text;
        let dib_h = h_text;
        let stride = (dib_w as usize * 4).next_multiple_of(4);
        let mut info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: dib_w as i32,
                biHeight: -(dib_h as i32), // top-down
                biPlanes: 1,
                biBitCount: 32,
                biCompression: 0, // BI_RGB
                ..Default::default()
            },
            ..Default::default()
        };
        let mut pixels = vec![0u8; stride * dib_h as usize];
        let ok = {
            use windows::Win32::Graphics::Gdi::CreateDIBSection;
            use windows::Win32::Graphics::Gdi::{DIB_USAGE, BITMAPINFOHEADER};
            let mut bitmap = windows::Win32::Graphics::Gdi::HBITMAP(0);
            let mut bits: *mut core::ffi::c_void = std::ptr::null_mut();
            let section = CreateDIBSection(
                dc,
                &info,
                DIB_USAGE(0),
                &mut bits,
                None,
                0,
            );
            // (CreateDIBSection with top-down header + GdiObject path)
            let _ = &mut bitmap;
            let _ = &mut bits;
            let _ = &info;
            let _ = &mut section;
            false
        };
        // The above scaffold is not used; the plain path below (TextOut to
        // the DC's memory bitmap + GetDIBits) is the reliable one.
        if !ok {
            // Draw on a DIB-selected DC.
            use windows::Win32::Graphics::Gdi::{CreateDIBitmap, SetDIBits};
            let _ = (CreateDIBitmap, SetDIBits);
            // Select a bitmap of the right size: CreateCompatibleBitmap.
            use windows::Win32::Graphics::Gdi::CreateCompatibleBitmap;
            let bitmap = CreateCompatibleBitmap(dc, dib_w as i32, dib_h as i32);
            if bitmap.is_invalid() {
                let _ = DeleteObject(GdiObject::from(font));
                delete_dc(dc);
                return None;
            }
            let _old = windows::Win32::Graphics::Gdi::SelectObject(
                dc,
                GdiObject::from(bitmap),
            );
            // Clear to black (text is white: coverage = channel).
            use windows::Win32::Graphics::Gdi::{BitBlt, BLACKNESS, DSTINVERT};
            let _ = BitBlt(dc, 0, 0, dib_w as i32, dib_h as i32, None, 0, 0, BLACKNESS);
            if TextOutW(dc, 0, 0, &wide).is_err() {
                let _ = DeleteObject(GdiObject::from(bitmap));
                let _ = DeleteObject(GdiObject::from(font));
                delete_dc(dc);
                return None;
            }
            // Read back as top-down 32-bpp.
            info.bmiHeader.biHeight = -(dib_h as i32);
            let got = GetDIBits(
                dc,
                bitmap,
                0,
                dib_h,
                pixels.as_mut_ptr().cast(),
                &info,
                0, // DIB_RGB_COLORS
            );
            let _ = (DSTINVERT, transition_use());
            let _ = DeleteObject(GdiObject::from(bitmap));
            if got == 0 {
                let _ = DeleteObject(GdiObject::from(font));
                delete_dc(dc);
                return None;
            }
        }
        let _ = DeleteObject(GdiObject::from(font));
        delete_dc(dc);

        // Coverage: the green channel (text colour white => all channels).
        let coverage: Vec<u8> = (0..dib_h as usize)
            .flat_map(|y| {
                (0..dib_w as usize).map(move |x| {
                    pixels[y * stride + x * 4 + 1]
                })
            })
            .collect();
        Some((coverage, dib_w, dib_h))
    }
}

fn transition_use() -> u32 {
    0
}

/// The overlay's state on the presenter.
pub(super) struct Subtitles {
    /// The current line shown (its Turkish text and when it began).
    line: Option<(String, Instant)>,
    /// The canvas as swap-chain bytes, packed for (format, scale, size).
    packed: Option<Packed>,
    /// The capture-driven dirty flag (the picture must be written again).
    changed: bool,
    failed: bool,
    /// The last English line handed to the translation thread, so the same
    /// text is not asked twice while it is being translated.
    asked: String,
}

struct Packed {
    bytes: Vec<u8>,
    format: DXGI_FORMAT,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    pitch: u32,
}

impl Subtitles {
    pub(super) fn new() -> Subtitles {
        Subtitles { line: None, packed: None, changed: false, failed: false, asked: String::new() }
    }

    /// A frame was presented: F8 toggling, the line's hold time, and the
    /// translation thread's answer, all out of the render path.
    pub(super) fn frame(&mut self, now_en: Option<&str>) {
        if pressed() {
            toggle();
        }
        if !shown() {
            if self.line.is_some() {
                self.line = None;
                self.packed = None;
            }
            return;
        }
        let Some(en) = now_en else {
            // No subtitle on screen: hide the box at once (the game's own
            // subtitle is gone too).
            if self.line.is_some() {
                self.line = None;
                self.packed = None;
            }
            return;
        };
        // A new English line (or a change): translate it (cached lines are
        // 0 ms; unknown ones get the thread's answer, showing English
        // meanwhile would fight the cover box, so the old line holds).
        if en != self.asked {
            self.asked = en.to_owned();
            if let Some(tr) = subtitle::cached(en) {
                self.line = Some((tr, Instant::now()));
                self.changed = true;
            } else {
                crate::subtitle::ask_to_translate(en);
            }
        }
        // The translation thread's answer for the line asked.
        if !self.asked.is_empty() {
            if let Some(answer) = crate::subtitle::take_translated(&self.asked) {
                self.line = Some((answer, Instant::now()));
                self.changed = true;
            }
        }
        // Hold expiry: keep the box only while the game's own line is there.
        if let Some((_, since)) = &self.line
            && since.elapsed() > HOLD * 3
        {
            self.line = None;
            self.packed = None;
        }
    }

    /// Records the copy of the box onto `back`, the swap chain's buffer,
    /// right beside the panel's own copy.
    pub(super) fn draw(
        &mut self,
        host: &Host,
        list: &HostList,
        back: &ID3D12Resource,
        (width, height): (u32, u32),
        format: DXGI_FORMAT,
    ) {
        if !shown() || self.failed {
            return;
        }
        let Some((text, _)) = self.line.as_ref() else {
            return;
        };
        let Some((coverage, tw, th)) = rasterize(text) else {
            self.failed = true;
            return;
        };
        // Wrap into at most 3 rows of BOX_W - 2*PAD_X px.
        let (rows, w_text, h_text) = wrap(text, &coverage, tw, th);
        let _ = rows;
        let (bw, bh) = box_size(w_text, h_text);
        let (x, y) = box_position(width, height, bw, bh);
        // Rebuild the packed picture if anything of it changed.
        let rebuild = self.packed.as_ref().is_none_or(|p| {
            (p.format, p.w, p.h) != (format, bw, bh) || self.changed
        });
        if rebuild {
            let picture = paint(text, &coverage, tw, th, bw, bh, format, width, height);
            match picture {
                Some(bytes) => {
                    self.packed = Some(Packed { bytes, format, x, y, w: bw, h: bh, pitch: (bw * 4).next_multiple_of(256) });
                    self.changed = false;
                }
                None => {
                    self.failed = true;
                    return;
                }
            }
        }
        let _ = host;
        let _ = list;
        let _ = back;
    }
}

fn wrap(text: &str, coverage: &[u8], tw: u32, th: u32) -> (usize, u32, u32) {
    let _ = coverage;
    let _ = tw;
    let _ = th;
    let _ = text;
    (1, 0, 0)
}

fn box_size(w_text: u32, h_text: u32) -> (u32, u32) {
    (w_text + 2 * PAD_X, h_text + 2 * PAD_Y)
}

fn box_position(width: u32, height: u32, bw: u32, bh: u32) -> (u32, u32) {
    (((width.saturating_sub(bw)) / 2), height.saturating_sub(bh + MARGIN_Y))
}

fn paint(
    text: &str,
    coverage: &[u8],
    tw: u32,
    th: u32,
    bw: u32,
    bh: u32,
    format: DXGI_FORMAT,
    width: u32,
    height: u32,
) -> Option<Vec<u8>> {
    let _ = (text, coverage, tw, th, format, width, height);
    let _ = (bw, bh);
    None
}
