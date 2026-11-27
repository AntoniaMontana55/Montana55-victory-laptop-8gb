//! Live Turkish subtitle overlay: a cinematic rounded black box at the bottom
//! center of the screen covering the game's English subtitles — near-opaque
//! anti-aliased rounded background, horizontally centred white Segoe UI
//! Semibold text with full Turkish glyph support (`ğ, ü, ş, ı, İ, ö, ç, â, û`),
//! toggled on/off at runtime with F8 and alpha-composited onto the swap chain's
//! back buffer right before Present.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use windows::Win32::Foundation::{COLORREF, RECT, SIZE};
use windows::Win32::Graphics::Direct3D::D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST;
use windows::Win32::Graphics::Direct3D12::*;
use windows::Win32::Graphics::Dxgi::Common::*;
use windows::Win32::Graphics::Gdi::{
    BITMAPINFO, BITMAPINFOHEADER, CLEARTYPE_QUALITY, CreateCompatibleDC, CreateDIBSection,
    CreateFontW, DEFAULT_CHARSET, DIB_RGB_COLORS, DeleteDC, DeleteObject, FONT_CLIP_PRECISION,
    FONT_OUTPUT_PRECISION, FW_SEMIBOLD, GetTextExtentPoint32W, HGDIOBJ, SelectObject, SetBkMode,
    SetTextColor, TRANSPARENT, TextOutW,
};
use windows::core::{s, w};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_F8};
use windows_sys::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

use super::ocr::Capturer;
use crate::host::{Host, HostList, transition};
use crate::subtitle;

/// Maximum wrapped subtitle rows.
const MAX_LINES: usize = 3;
/// Number of upload ring slots (matches `host::panel`, safe across 2+ in-flight frames).
const UPLOADS: usize = 4;
/// Near-opaque cinematic black box alpha (~91%, covers underlying English subtitle cleanly).
const BG_ALPHA: f32 = 0.91;
/// Subtle inner rim brightness (sleek cinema box edge).
const RIM_RGB: f32 = 52.0;
const RIM_ALPHA: f32 = 0.45;

/// Grace period before hiding if OCR stops producing new updates on an unchanged line.
/// While OCR still actively sees the same English line on screen, the hold refreshes.
const HOLD_GRACE: Duration = Duration::from_secs(8);

/// Shared F8 toggle flag (initialized from `VICTORY_SUBS`).
fn shown_flag() -> &'static AtomicBool {
    static SHOWN: OnceLock<AtomicBool> = OnceLock::new();
    SHOWN.get_or_init(|| AtomicBool::new(subtitle::asked()))
}

fn shown() -> bool {
    shown_flag().load(Ordering::Relaxed)
}

/// Rising-edge F8 check while one of our windows has foreground focus.
fn pressed() -> bool {
    static DOWN: AtomicBool = AtomicBool::new(false);
    // SAFETY: plain Win32 key-state query.
    let down = unsafe { GetAsyncKeyState(i32::from(VK_F8)) } < 0;
    if DOWN.swap(down, Ordering::Relaxed) || !down {
        return false;
    }
    let mut process = 0;
    // SAFETY: plain window query; room for process id.
    unsafe { GetWindowThreadProcessId(GetForegroundWindow(), &mut process) };
    process == std::process::id()
}

/// Resolution-scaled typography & box geometry metrics.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Metrics {
    font_h: u32,
    line_h: u32,
    pad_x: u32,
    pad_y: u32,
    radius: u32,
    margin_y: u32,
    max_box_w: u32,
    max_text_w: u32,
}

impl Metrics {
    fn for_screen(width: u32, height: u32) -> Self {
        let scale = (height as f32 / 1080.0).clamp(0.65, 2.0);
        let font_h = (30.0 * scale).round().max(18.0) as u32;
        let line_h = (40.0 * scale).round().max(24.0) as u32;
        let pad_x = (30.0 * scale).round().max(16.0) as u32;
        let pad_y = (16.0 * scale).round().max(10.0) as u32;
        let radius = (12.0 * scale).round().max(6.0) as u32;
        let margin_y = (44.0 * scale).round().max(24.0) as u32;
        let max_box_w = ((1160.0 * scale).round() as u32)
            .min(width.saturating_mul(82) / 100)
            .max(240)
            .min(width.max(1));
        let max_text_w = max_box_w.saturating_sub(2 * pad_x).max(120);
        Self { font_h, line_h, pad_x, pad_y, radius, margin_y, max_box_w, max_text_w }
    }
}

/// Wraps and rasterizes `text` into a tightly fitted centered coverage mask
/// `(coverage, text_w, text_h)` using GDI Segoe UI Semibold (full Turkish Unicode).
fn rasterize_text(text: &str, m: Metrics) -> Option<(Vec<u8>, u32, u32)> {
    // SAFETY: self-contained GDI offscreen DC, font, and top-down 32-bpp DIB section;
    // all objects are deselected and deleted before returning.
    unsafe {
        let dc = CreateCompatibleDC(None);
        if dc.is_invalid() {
            return None;
        }
        let font = CreateFontW(
            -(m.font_h as i32),
            0,
            0,
            0,
            FW_SEMIBOLD.0 as i32,
            0,
            0,
            0,
            DEFAULT_CHARSET,
            FONT_OUTPUT_PRECISION(0),
            FONT_CLIP_PRECISION(0),
            CLEARTYPE_QUALITY,
            0,
            w!("Segoe UI"),
        );
        if font.is_invalid() {
            let _ = DeleteDC(dc);
            return None;
        }
        let old_font = SelectObject(dc, HGDIOBJ::from(font));

        let measure = |s: &str| -> (u32, u32) {
            if s.is_empty() {
                return (0, m.line_h);
            }
            let wide: Vec<u16> = s.encode_utf16().collect();
            let mut sz = SIZE::default();
            if GetTextExtentPoint32W(dc, &wide, &mut sz).as_bool() {
                (sz.cx.max(1) as u32, sz.cy.max(1) as u32)
            } else {
                ((s.chars().count() as u32) * (m.font_h / 2).max(8), m.line_h)
            }
        };

        let lines = wrap_lines(text, m.max_text_w, &measure);
        if lines.is_empty() {
            SelectObject(dc, old_font);
            let _ = DeleteObject(HGDIOBJ::from(font));
            let _ = DeleteDC(dc);
            return None;
        }

        let measured: Vec<(String, u32, u32)> = lines
            .into_iter()
            .map(|line| {
                let (w, h) = measure(&line);
                (line, w, h)
            })
            .collect();
        let text_w = measured.iter().map(|l| l.1).max().unwrap_or(1).min(m.max_text_w).max(1);
        let row_h = measured.iter().map(|l| l.2).max().unwrap_or(m.font_h).max(m.line_h);
        let text_h = (row_h * measured.len() as u32).max(1);

        let info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: text_w as i32,
                biHeight: -(text_h as i32), // top-down
                biPlanes: 1,
                biBitCount: 32,
                biCompression: 0, // BI_RGB
                ..Default::default()
            },
            ..Default::default()
        };
        let mut bits: *mut core::ffi::c_void = std::ptr::null_mut();
        let Ok(bitmap) = CreateDIBSection(Some(dc), &info, DIB_RGB_COLORS, &mut bits, None, 0) else {
            SelectObject(dc, old_font);
            let _ = DeleteObject(HGDIOBJ::from(font));
            let _ = DeleteDC(dc);
            return None;
        };
        if bitmap.is_invalid() || bits.is_null() {
            if !bitmap.is_invalid() {
                let _ = DeleteObject(HGDIOBJ::from(bitmap));
            }
            SelectObject(dc, old_font);
            let _ = DeleteObject(HGDIOBJ::from(font));
            let _ = DeleteDC(dc);
            return None;
        }

        let byte_len = text_w as usize * text_h as usize * 4;
        std::ptr::write_bytes(bits.cast::<u8>(), 0, byte_len);

        let old_bmp = SelectObject(dc, HGDIOBJ::from(bitmap));
        let _ = SetBkMode(dc, TRANSPARENT);
        let _ = SetTextColor(dc, COLORREF(0x00FF_FFFF));

        for (row_idx, (line, lw, lh)) in measured.iter().enumerate() {
            let wide: Vec<u16> = line.encode_utf16().collect();
            let lx = (text_w.saturating_sub(*lw) / 2) as i32;
            let ly = (row_idx as u32 * row_h + row_h.saturating_sub(*lh) / 2) as i32;
            let _ = TextOutW(dc, lx, ly, &wide);
        }

        let pixels = std::slice::from_raw_parts(bits.cast::<u8>(), byte_len);
        let mut coverage = vec![0u8; text_w as usize * text_h as usize];
        for (i, px) in pixels.chunks_exact(4).enumerate() {
            // ClearType renders RGB subpixels on black: weighted luminance keeps smooth stems.
            let b = u16::from(px[0]);
            let g = u16::from(px[1]);
            let r = u16::from(px[2]);
            coverage[i] = ((r + g * 2 + b) / 4).max(g) as u8;
        }

        SelectObject(dc, old_bmp);
        SelectObject(dc, old_font);
        let _ = DeleteObject(HGDIOBJ::from(bitmap));
        let _ = DeleteObject(HGDIOBJ::from(font));
        let _ = DeleteDC(dc);

        Some((coverage, text_w, text_h))
    }
}

/// Word-wraps `text` into at most [`MAX_LINES`] lines fitting `max_w`, using
/// `measure` for exact proportional widths.
fn wrap_lines(text: &str, max_w: u32, measure: &impl Fn(&str) -> (u32, u32)) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if current.is_empty() {
            current.push_str(word);
            continue;
        }
        let mut candidate = String::with_capacity(current.len() + 1 + word.len());
        candidate.push_str(&current);
        candidate.push(' ');
        candidate.push_str(word);
        if measure(&candidate).0 <= max_w {
            current = candidate;
        } else {
            lines.push(std::mem::take(&mut current));
            if lines.len() >= MAX_LINES {
                if let Some(last) = lines.last_mut()
                    && !last.ends_with("...")
                {
                    last.push_str("...");
                }
                return lines;
            }
            current.push_str(word);
        }
    }
    if !current.is_empty() && lines.len() < MAX_LINES {
        lines.push(current);
    }
    lines
}

/// Paints a premultiplied-alpha RGBA8 rounded black box with subtle drop shadow
/// and centered white text (`bw x bh`).
fn paint_box(
    coverage: &[u8],
    text_w: u32,
    text_h: u32,
    m: Metrics,
) -> (Vec<u8>, u32, u32) {
    let bw = (text_w + 2 * m.pad_x).min(m.max_box_w).max(2 * m.radius + 4);
    let bh = (text_h + 2 * m.pad_y).max(2 * m.radius + 4);
    let mut rgba = vec![0u8; bw as usize * bh as usize * 4];

    let half_w = bw as f32 * 0.5;
    let half_h = bh as f32 * 0.5;
    let r = (m.radius as f32).min(half_w - 1.0).min(half_h - 1.0);
    let inner_x = half_w - r;
    let inner_y = half_h - r;

    let ox = bw.saturating_sub(text_w) / 2;
    let oy = bh.saturating_sub(text_h) / 2;

    let sample_cov = |x: i32, y: i32| -> f32 {
        if x < 0 || y < 0 || x >= text_w as i32 || y >= text_h as i32 {
            0.0
        } else {
            f32::from(coverage[y as usize * text_w as usize + x as usize]) / 255.0
        }
    };

    for y in 0..bh {
        let py = (y as f32 + 0.5 - half_h).abs();
        let dy = py - inner_y;
        let ty = y as i32 - oy as i32;
        for x in 0..bw {
            let px = (x as f32 + 0.5 - half_w).abs();
            let dx = px - inner_x;
            // Exact SDF of a rounded rectangle.
            let dist = (dx.max(0.0).hypot(dy.max(0.0))) + dx.max(dy).min(0.0) - r;
            let box_mask = (0.5 - dist).clamp(0.0, 1.0);
            if box_mask <= 0.0 {
                continue;
            }
            let tx = x as i32 - ox as i32;
            let glyph = sample_cov(tx, ty);
            let shadow = sample_cov(tx - 1, ty - 1) * 0.55;

            // Subtle 1 px rim along the inside of the rounded outline.
            let rim = if dist < -0.15 && dist > -1.4 {
                RIM_ALPHA * (1.0 - (dist + 0.75).abs() / 0.65).clamp(0.0, 1.0)
            } else {
                0.0
            };

            let bg_a = (BG_ALPHA + shadow * (1.0 - BG_ALPHA)).clamp(0.0, 1.0);
            let out_a = (bg_a + glyph * (1.0 - bg_a)) * box_mask;
            // Premultiplied RGB: pure white glyph + faint rim over black backdrop.
            let rgb = (RIM_RGB * rim * (1.0 - glyph) + 255.0 * glyph) * box_mask;
            let c = rgb.round().clamp(0.0, 255.0) as u8;
            let a = (out_a * 255.0).round().clamp(0.0, 255.0) as u8;
            let at = (y as usize * bw as usize + x as usize) * 4;
            rgba[at] = c;
            rgba[at + 1] = c;
            rgba[at + 2] = c;
            rgba[at + 3] = a;
        }
    }
    (rgba, bw, bh)
}

fn box_position(width: u32, height: u32, bw: u32, bh: u32, margin_y: u32) -> (u32, u32) {
    ((width.saturating_sub(bw)) / 2, height.saturating_sub(bh + margin_y))
}

/// Premultiplied-alpha fullscreen-in-viewport triangle pass (same lightweight
/// pattern as `compose.rs` and `host/present_planes.rs`).
const OVERLAY_HLSL: &str = r#"
#define RS "DescriptorTable(SRV(t0), visibility = SHADER_VISIBILITY_PIXEL), RootConstants(num32BitConstants = 4, b0, visibility = SHADER_VISIBILITY_PIXEL), StaticSampler(s0, filter = FILTER_MIN_MAG_MIP_LINEAR, addressU = TEXTURE_ADDRESS_CLAMP, addressV = TEXTURE_ADDRESS_CLAMP, addressW = TEXTURE_ADDRESS_CLAMP, visibility = SHADER_VISIBILITY_PIXEL)"
Texture2D<float4> box_tex : register(t0);
SamplerState lin : register(s0);
cbuffer Params : register(b0) { float4 uv_scale; };
struct V { float4 pos : SV_Position; float2 uv : TEXCOORD0; };
[RootSignature(RS)]
V vs(uint id : SV_VertexID) {
    V v;
    v.uv = float2(float((id << 1) & 2), float(id & 2));
    v.pos = float4(v.uv.x * 2.0 - 1.0, 1.0 - v.uv.y * 2.0, 0.0, 1.0);
    return v;
}
[RootSignature(RS)]
float4 ps(V v) : SV_Target {
    return box_tex.SampleLevel(lin, v.uv * uv_scale.xy, 0);
}
"#;

struct OverlayPass {
    vs: Vec<u8>,
    ps: Vec<u8>,
    root: ID3D12RootSignature,
    psos: Mutex<Vec<(i32, Option<ID3D12PipelineState>)>>,
}

// SAFETY: D3D12 root signatures and PSOs are free-threaded.
unsafe impl Send for OverlayPass {}
unsafe impl Sync for OverlayPass {}

fn overlay_pass(device: &ID3D12Device) -> Option<&'static OverlayPass> {
    static PASS: OnceLock<Option<OverlayPass>> = OnceLock::new();
    PASS.get_or_init(|| {
        let make = || -> windows::core::Result<OverlayPass> {
            let vs = crate::host::compile(OVERLAY_HLSL, s!("vs"), s!("vs_5_1"))?;
            let ps = crate::host::compile(OVERLAY_HLSL, s!("ps"), s!("ps_5_1"))?;
            // SAFETY: compiled shader carries serialized root signature.
            let root = unsafe { device.CreateRootSignature(0, &ps)? };
            Ok(OverlayPass { vs, ps, root, psos: Mutex::new(Vec::new()) })
        };
        make()
            .inspect_err(|e| tracing::warn!(error = %e, "subtitles: overlay pass compile failed"))
            .ok()
    })
    .as_ref()
}

impl OverlayPass {
    fn pipeline(&self, device: &ID3D12Device, format: DXGI_FORMAT) -> Option<ID3D12PipelineState> {
        let mut psos = self.psos.lock().unwrap_or_else(|e| e.into_inner());
        if let Some((_, pso)) = psos.iter().find(|(f, _)| *f == format.0) {
            return pso.clone();
        }
        let code = |b: &[u8]| D3D12_SHADER_BYTECODE {
            pShaderBytecode: b.as_ptr().cast(),
            BytecodeLength: b.len(),
        };
        let mut blend = D3D12_BLEND_DESC::default();
        blend.RenderTarget[0] = D3D12_RENDER_TARGET_BLEND_DESC {
            BlendEnable: true.into(),
            // Premultiplied alpha: smooth anti-aliased rounded corners & crisp text.
            SrcBlend: D3D12_BLEND_ONE,
            DestBlend: D3D12_BLEND_INV_SRC_ALPHA,
            BlendOp: D3D12_BLEND_OP_ADD,
            SrcBlendAlpha: D3D12_BLEND_ONE,
            DestBlendAlpha: D3D12_BLEND_INV_SRC_ALPHA,
            BlendOpAlpha: D3D12_BLEND_OP_ADD,
            RenderTargetWriteMask: D3D12_COLOR_WRITE_ENABLE_ALL.0 as u8,
            ..Default::default()
        };
        let mut rtv_formats = [DXGI_FORMAT_UNKNOWN; 8];
        rtv_formats[0] = format;
        // SAFETY: descriptor pointers live through CreateGraphicsPipelineState.
        let pso = unsafe {
            device.CreateGraphicsPipelineState::<ID3D12PipelineState>(&D3D12_GRAPHICS_PIPELINE_STATE_DESC {
                pRootSignature: std::mem::transmute_copy(&self.root),
                VS: code(&self.vs),
                PS: code(&self.ps),
                BlendState: blend,
                SampleMask: u32::MAX,
                RasterizerState: D3D12_RASTERIZER_DESC {
                    FillMode: D3D12_FILL_MODE_SOLID,
                    CullMode: D3D12_CULL_MODE_NONE,
                    ..Default::default()
                },
                PrimitiveTopologyType: D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE,
                NumRenderTargets: 1,
                RTVFormats: rtv_formats,
                SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
                ..Default::default()
            })
        }
        .inspect_err(|e| tracing::warn!(error = %e, format = format.0, "subtitles: overlay PSO failed"))
        .ok();
        psos.push((format.0, pso.clone()));
        pso
    }
}

struct OverlayGpu {
    cap_w: u32,
    cap_h: u32,
    pitch: u32,
    tex: ID3D12Resource,
    srv_heap: ID3D12DescriptorHeap,
    uploads: Vec<(ID3D12Resource, *mut u8)>,
    turn: usize,
    /// Current uploaded box dimensions inside `tex`.
    cur_w: u32,
    cur_h: u32,
    fresh: bool,
    uploaded: bool,
}

// SAFETY: D3D12 objects are free-threaded; upload memory written only under `Presenter`'s lock.
unsafe impl Send for OverlayGpu {}

impl OverlayGpu {
    fn new(host: &Host, cap_w: u32, cap_h: u32) -> Option<Self> {
        let cap_w = cap_w.max(256);
        let cap_h = cap_h.max(64);
        let pitch = (cap_w * 4).next_multiple_of(256);
        let upload_bytes = u64::from(pitch) * u64::from(cap_h);
        let uploads = (0..UPLOADS)
            .map(|_| host.upload_buffer(upload_bytes))
            .collect::<Option<Vec<_>>>()?;
        let heap = D3D12_HEAP_PROPERTIES {
            Type: D3D12_HEAP_TYPE_DEFAULT,
            ..Default::default()
        };
        let desc = D3D12_RESOURCE_DESC {
            Dimension: D3D12_RESOURCE_DIMENSION_TEXTURE2D,
            Width: u64::from(cap_w),
            Height: cap_h,
            DepthOrArraySize: 1,
            MipLevels: 1,
            Format: DXGI_FORMAT_R8G8B8A8_UNORM,
            SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
            ..Default::default()
        };
        let mut tex: Option<ID3D12Resource> = None;
        // SAFETY: plain default-heap 2D texture + 1-slot shader-visible SRV heap.
        unsafe {
            host.device
                .CreateCommittedResource(
                    &heap,
                    D3D12_HEAP_FLAG_NONE,
                    &desc,
                    D3D12_RESOURCE_STATE_COPY_DEST,
                    None,
                    &mut tex,
                )
                .ok()?;
            let tex = tex?;
            let srv_heap: ID3D12DescriptorHeap = host
                .device
                .CreateDescriptorHeap(&D3D12_DESCRIPTOR_HEAP_DESC {
                    Type: D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV,
                    NumDescriptors: 1,
                    Flags: D3D12_DESCRIPTOR_HEAP_FLAG_SHADER_VISIBLE,
                    NodeMask: 0,
                })
                .ok()?;
            let srv = D3D12_SHADER_RESOURCE_VIEW_DESC {
                Format: DXGI_FORMAT_R8G8B8A8_UNORM,
                ViewDimension: D3D12_SRV_DIMENSION_TEXTURE2D,
                Shader4ComponentMapping: D3D12_DEFAULT_SHADER_4_COMPONENT_MAPPING,
                Anonymous: D3D12_SHADER_RESOURCE_VIEW_DESC_0 {
                    Texture2D: D3D12_TEX2D_SRV { MipLevels: 1, ..Default::default() },
                },
            };
            host.device.CreateShaderResourceView(
                Some(&tex),
                Some(&srv),
                srv_heap.GetCPUDescriptorHandleForHeapStart(),
            );
            Some(Self {
                cap_w,
                cap_h,
                pitch,
                tex,
                srv_heap,
                uploads,
                turn: 0,
                cur_w: 0,
                cur_h: 0,
                fresh: false,
                uploaded: false,
            })
        }
    }

    fn stage(&mut self, rgba: &[u8], bw: u32, bh: u32) {
        self.turn = (self.turn + 1) % UPLOADS;
        let dst_ptr = self.uploads[self.turn].1;
        let row_bytes = bw as usize * 4;
        let pitch = self.pitch as usize;
        // SAFETY: `bw <= cap_w`, `bh <= cap_h`, `uploads[turn]` is mapped for `pitch * cap_h`.
        unsafe {
            for y in 0..bh as usize {
                let src = &rgba[y * row_bytes..(y + 1) * row_bytes];
                std::ptr::copy_nonoverlapping(src.as_ptr(), dst_ptr.add(y * pitch), row_bytes);
            }
        }
        self.cur_w = bw;
        self.cur_h = bh;
        self.fresh = true;
    }
}

/// The live Turkish subtitle overlay state owned by [`crate::host::Presenter`].
pub(crate) struct Subtitles {
    capturer: Capturer,
    /// Currently displayed Turkish text and when it last updated.
    line: Option<(String, Instant)>,
    /// Last English line handed to the translation thread.
    asked: String,
    /// Last rendered `(text, screen_w, screen_h)` to avoid re-rasterizing static text.
    rendered_key: Option<(String, u32, u32)>,
    gpu: Option<OverlayGpu>,
    failed: bool,
}

impl Subtitles {
    pub(crate) fn new() -> Self {
        if subtitle::asked() {
            subtitle::ensure_cache_file();
        }
        Self {
            capturer: Capturer::new(),
            line: None,
            asked: String::new(),
            rendered_key: None,
            gpu: None,
            failed: false,
        }
    }

    /// Records an asynchronous ROI strip capture from `back` (before our
    /// Turkish box paints over it). Returns the readback slot index, if any.
    pub(crate) fn capture_record(
        &mut self,
        host: &Host,
        copy: &HostList,
        back: &ID3D12Resource,
        size: (u32, u32),
        format: DXGI_FORMAT,
    ) -> Option<usize> {
        if !shown() || self.failed {
            return None;
        }
        self.capturer.record(host, copy, back, size, format)
    }

    /// Records the fence number `flown` for the captured readback slot.
    pub(crate) fn capture_submitted(&mut self, slot: Option<usize>, flown: u64) {
        self.capturer.submitted(slot, flown);
    }

    /// Called once per presented frame (off the GPU record path): handles F8
    /// toggling, polls the OCR worker & translation worker, and updates hold state.
    pub(crate) fn frame(&mut self) {
        if pressed() {
            let now_on = !shown_flag().fetch_xor(true, Ordering::Relaxed);
            tracing::info!(shown = now_on, "live Turkish subtitles toggled (F8)");
            if now_on {
                subtitle::ensure_cache_file();
            } else {
                self.line = None;
                self.asked.clear();
                self.rendered_key = None;
            }
        }
        if !shown() {
            return;
        }
        let now_en = super::ocr::current_en();
        let Some(en) = now_en else {
            if self.line.is_some() {
                self.line = None;
                self.asked.clear();
            }
            return;
        };
        if en != self.asked {
            self.asked = en.clone();
            if let Some(tr) = subtitle::cached(&en) {
                if !tr.is_empty() {
                    self.line = Some((tr, Instant::now()));
                }
            } else {
                subtitle::ask_to_translate(&en);
            }
        } else if let Some((_, since)) = self.line.as_mut() {
            // OCR still sees the same English subtitle on screen: keep the box alive.
            *since = Instant::now();
        }
        if !self.asked.is_empty()
            && let Some(tr) = subtitle::take_translated(&self.asked)
            && !tr.is_empty()
        {
            self.line = Some((tr, Instant::now()));
        }
        if let Some((_, since)) = &self.line
            && since.elapsed() > HOLD_GRACE
        {
            self.line = None;
        }
    }

    /// Alpha-blends the rounded black box + centred white Turkish text onto
    /// `back` (which starts and ends in `D3D12_RESOURCE_STATE_COPY_DEST`).
    pub(crate) fn draw(
        &mut self,
        host: &Host,
        copy: &HostList,
        back: &ID3D12Resource,
        (width, height): (u32, u32),
        format: DXGI_FORMAT,
    ) {
        if !shown() || self.failed || width < 320 || height < 240 {
            return;
        }
        let Some((text, _)) = self.line.as_ref() else {
            return;
        };
        #[cfg(feature = "diagnostics")]
        let _ours = crate::replay::record::unrecorded();

        let Some(pass) = overlay_pass(&host.device) else {
            self.failed = true;
            return;
        };
        let Some(pso) = pass.pipeline(&host.device, format) else {
            return;
        };

        let m = Metrics::for_screen(width, height);
        let need_rebuild = self
            .rendered_key
            .as_ref()
            .is_none_or(|(t, w, h)| t != text || *w != width || *h != height);

        if need_rebuild {
            let Some((cov, tw, th)) = rasterize_text(text, m) else {
                return;
            };
            let (rgba, bw, bh) = paint_box(&cov, tw, th, m);
            let need_gpu = self.gpu.as_ref().is_none_or(|g| bw > g.cap_w || bh > g.cap_h);
            if need_gpu {
                if let Some(old) = self.gpu.take() {
                    crate::host::retire_resource(old.tex);
                    crate::host::retire(old.srv_heap);
                    for (buf, _) in old.uploads {
                        crate::host::retire_resource(buf);
                    }
                }
                let cap_w = bw.max(m.max_box_w).next_multiple_of(64);
                let cap_h = bh.max(m.line_h * MAX_LINES as u32 + 2 * m.pad_y + 16).next_multiple_of(32);
                self.gpu = OverlayGpu::new(host, cap_w, cap_h);
                if self.gpu.is_none() {
                    self.failed = true;
                    return;
                }
            }
            if let Some(gpu) = self.gpu.as_mut() {
                gpu.stage(&rgba, bw, bh);
                self.rendered_key = Some((text.clone(), width, height));
            }
        }

        let Some(gpu) = self.gpu.as_mut() else {
            return;
        };
        if gpu.cur_w == 0 || gpu.cur_h == 0 {
            return;
        }

        let (bx, by) = box_position(width, height, gpu.cur_w, gpu.cur_h, m.margin_y);
        copy.keep(&gpu.tex);
        copy.keep(&gpu.srv_heap);
        let list = &copy.list;

        // SAFETY: `list` is open; `back` starts in `COPY_DEST` and is restored to
        // `COPY_DEST`; `gpu.tex`, `gpu.srv_heap` and `gpu.uploads[turn]` stay live.
        unsafe {
            if std::mem::take(&mut gpu.fresh) {
                copy.keep(&gpu.uploads[gpu.turn].0);
                if gpu.uploaded {
                    list.ResourceBarrier(&[transition(
                        &gpu.tex,
                        D3D12_RESOURCE_STATE_PIXEL_SHADER_RESOURCE,
                        D3D12_RESOURCE_STATE_COPY_DEST,
                    )]);
                }
                let from = D3D12_TEXTURE_COPY_LOCATION {
                    pResource: std::mem::transmute_copy(&gpu.uploads[gpu.turn].0),
                    Type: D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT,
                    Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
                        PlacedFootprint: D3D12_PLACED_SUBRESOURCE_FOOTPRINT {
                            Offset: 0,
                            Footprint: D3D12_SUBRESOURCE_FOOTPRINT {
                                Format: DXGI_FORMAT_R8G8B8A8_UNORM,
                                Width: gpu.cur_w,
                                Height: gpu.cur_h,
                                Depth: 1,
                                RowPitch: gpu.pitch,
                            },
                        },
                    },
                };
                let to = D3D12_TEXTURE_COPY_LOCATION {
                    pResource: std::mem::transmute_copy(&gpu.tex),
                    Type: D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
                    Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 { SubresourceIndex: 0 },
                };
                let part = D3D12_BOX {
                    left: 0,
                    top: 0,
                    front: 0,
                    right: gpu.cur_w,
                    bottom: gpu.cur_h,
                    back: 1,
                };
                list.CopyTextureRegion(&to, 0, 0, 0, &from, Some(&part));
                list.ResourceBarrier(&[transition(
                    &gpu.tex,
                    D3D12_RESOURCE_STATE_COPY_DEST,
                    D3D12_RESOURCE_STATE_PIXEL_SHADER_RESOURCE,
                )]);
                gpu.uploaded = true;
            }
            if !gpu.uploaded {
                return;
            }
            list.ResourceBarrier(&[transition(
                back,
                D3D12_RESOURCE_STATE_COPY_DEST,
                D3D12_RESOURCE_STATE_RENDER_TARGET,
            )]);
            let rtv = host.rtv(Some(back), None);
            let uv = [
                gpu.cur_w as f32 / gpu.cap_w as f32,
                gpu.cur_h as f32 / gpu.cap_h as f32,
                0.0f32,
                0.0f32,
            ];
            list.OMSetRenderTargets(1, Some(&rtv), false, None);
            list.SetDescriptorHeaps(&[Some(gpu.srv_heap.clone())]);
            list.SetGraphicsRootSignature(&pass.root);
            list.SetPipelineState(&pso);
            list.SetGraphicsRootDescriptorTable(0, gpu.srv_heap.GetGPUDescriptorHandleForHeapStart());
            list.SetGraphicsRoot32BitConstants(1, 4, uv.as_ptr().cast(), 0);
            list.IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
            list.RSSetViewports(&[D3D12_VIEWPORT {
                TopLeftX: bx as f32,
                TopLeftY: by as f32,
                Width: gpu.cur_w as f32,
                Height: gpu.cur_h as f32,
                MinDepth: 0.0,
                MaxDepth: 1.0,
            }]);
            list.RSSetScissorRects(&[RECT {
                left: bx as i32,
                top: by as i32,
                right: (bx + gpu.cur_w).min(width) as i32,
                bottom: (by + gpu.cur_h).min(height) as i32,
            }]);
            list.DrawInstanced(3, 1, 0, 0);
            list.ResourceBarrier(&[transition(
                back,
                D3D12_RESOURCE_STATE_RENDER_TARGET,
                D3D12_RESOURCE_STATE_COPY_DEST,
            )]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounded_box_has_transparent_corners_and_opaque_center() {
        let m = Metrics::for_screen(1920, 1080);
        let cov = vec![255u8; 200 * 40];
        let (rgba, bw, bh) = paint_box(&cov, 200, 40, m);
        assert!(bw >= 200 + 2 * m.pad_x);
        assert!(bh >= 40 + 2 * m.pad_y);
        // Outer corner pixel (0, 0) lies outside the corner radius -> transparent.
        assert_eq!(rgba[3], 0);
        // Center pixel has white text over near-opaque black box.
        let cx = (bw / 2) as usize;
        let cy = (bh / 2) as usize;
        let at = (cy * bw as usize + cx) * 4;
        assert_eq!(rgba[at], 255);
        assert_eq!(rgba[at + 3], 255);
    }

    #[test]
    fn turkish_text_rasterizes_via_gdi() {
        let m = Metrics::for_screen(1920, 1080);
        let (cov, tw, th) = rasterize_text("Ne oluyor lan, ğüşıİöç!", m).expect("GDI rasterizes");
        assert!(tw > 40 && th >= m.line_h);
        assert!(cov.iter().any(|&v| v > 180), "rendered text has visible white glyphs");
    }

    #[test]
    fn word_wrap_limits_to_three_lines() {
        let measure = |s: &str| (s.len() as u32 * 10, 30);
        let lines = wrap_lines("bir iki üç dört beş altı yedi sekiz dokuz on onbir oniki", 120, &measure);
        assert!(lines.len() <= MAX_LINES);
        assert!(!lines.is_empty());
    }
}
