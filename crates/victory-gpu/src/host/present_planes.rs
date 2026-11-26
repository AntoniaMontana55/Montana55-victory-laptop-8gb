//! Composite Present for titles that submit separate game and UI surfaces.
use super::*;

pub(crate) struct PresentPlane {
    pub(crate) resource: usize,
    pub(crate) texture: std::sync::Arc<crate::resource::HostTexture>,
    pub(crate) src: Option<[i32; 4]>,
    pub(crate) dest: Option<[f32; 4]>,
}

/// The pass that composes present planes: a triangle over the placement,
/// the plane's source rectangle sampled bilinearly.
const COMPOSE: &str = r#"
#define RS "DescriptorTable(SRV(t0), visibility = SHADER_VISIBILITY_PIXEL), RootConstants(num32BitConstants = 4, b0, visibility = SHADER_VISIBILITY_PIXEL), StaticSampler(s0, filter = FILTER_MIN_MAG_MIP_LINEAR, addressU = TEXTURE_ADDRESS_CLAMP, addressV = TEXTURE_ADDRESS_CLAMP, addressW = TEXTURE_ADDRESS_CLAMP, visibility = SHADER_VISIBILITY_PIXEL)"
Texture2D<float4> plane : register(t0);
SamplerState clamped : register(s0);
cbuffer Source : register(b0) { float4 source; };
struct Corner { float4 p : SV_Position; float2 t : TEXCOORD0; };
Corner corner(uint i : SV_VertexID) {
    Corner c;
    c.t = float2(float(i & 1) * 2.0, float(i >> 1) * 2.0);
    c.p = float4(c.t.x * 2.0 - 1.0, 1.0 - c.t.y * 2.0, 0.0, 1.0);
    return c;
}
[RootSignature(RS)]
float4 compose(Corner c) : SV_Target {
    return plane.SampleLevel(clamped, lerp(source.xy, source.zw, c.t), 0);
}
"#;

struct ComposePass {
    vs: Vec<u8>,
    ps: Vec<u8>,
    root: ID3D12RootSignature,
    made: Mutex<Vec<(i32, u8, Option<ID3D12PipelineState>)>>,
}

fn compose_pass(device: &ID3D12Device) -> Option<&'static ComposePass> {
    static PASS: OnceLock<Option<ComposePass>> = OnceLock::new();
    PASS.get_or_init(|| {
        let make = || -> windows::core::Result<ComposePass> {
            use windows::core::s;
            let vs = compile(COMPOSE, s!("corner"), s!("vs_5_1"))?;
            let ps = compile(COMPOSE, s!("compose"), s!("ps_5_1"))?;
            // SAFETY: the pixel shader contains the serialized root signature.
            let root = unsafe { device.CreateRootSignature(0, &ps[..])? };
            Ok(ComposePass { vs, ps, root, made: Mutex::default() })
        };
        make().inspect_err(|e| tracing::warn!(error = %e, "no pass to compose present planes")).ok()
    }).as_ref()
}

impl ComposePass {
    /// The pipeline drawing into `format` with blend `blend`.
    fn pipeline(&self, device: &ID3D12Device, format: DXGI_FORMAT, blend: u8) -> Option<ID3D12PipelineState> {
        let mut made = self.made.lock().unwrap_or_else(|e| e.into_inner());
        if let Some((_, _, pipeline)) = made.iter().find(|m| (m.0, m.1) == (format.0, blend)) {
            return pipeline.clone();
        }
        let code = |bytes: &[u8]| D3D12_SHADER_BYTECODE {
            pShaderBytecode: bytes.as_ptr().cast(),
            BytecodeLength: bytes.len(),
        };
        let mut blend_state = D3D12_BLEND_DESC::default();
        blend_state.RenderTarget[0] = D3D12_RENDER_TARGET_BLEND_DESC {
            BlendEnable: (blend != 0).into(),
            SrcBlend: if blend == 2 { D3D12_BLEND_ONE } else { D3D12_BLEND_SRC_ALPHA },
            DestBlend: D3D12_BLEND_INV_SRC_ALPHA,
            BlendOp: D3D12_BLEND_OP_ADD,
            SrcBlendAlpha: D3D12_BLEND_ONE,
            DestBlendAlpha: D3D12_BLEND_INV_SRC_ALPHA,
            BlendOpAlpha: D3D12_BLEND_OP_ADD,
            RenderTargetWriteMask: 0xf,
            ..Default::default()
        };
        let mut formats = [DXGI_FORMAT_UNKNOWN; 8];
        formats[0] = format;
        // SAFETY: the description points at live data and never releases
        // its pointer.
        let pipeline = unsafe {
            device.CreateGraphicsPipelineState::<ID3D12PipelineState>(&D3D12_GRAPHICS_PIPELINE_STATE_DESC {
                pRootSignature: std::mem::transmute_copy(&self.root),
                VS: code(&self.vs),
                PS: code(&self.ps),
                BlendState: blend_state,
                SampleMask: u32::MAX,
                RasterizerState: D3D12_RASTERIZER_DESC {
                    FillMode: D3D12_FILL_MODE_SOLID,
                    CullMode: D3D12_CULL_MODE_NONE,
                    DepthClipEnable: true.into(),
                    ..Default::default()
                },
                PrimitiveTopologyType: D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE,
                NumRenderTargets: 1,
                RTVFormats: formats,
                SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
                ..Default::default()
            })
        }
        .inspect_err(|e| tracing::warn!(error = %e, format = format.0, blend, "no pipeline to compose present planes"))
        .ok();
        made.push((format.0, blend, pipeline.clone()));
        pipeline
    }
}

/// The format a plane of `format` is read with: typeless ones as their
/// UNORM or float member; `None` for what a float sampler cannot read.
fn view_format(format: DXGI_FORMAT) -> Option<DXGI_FORMAT> {
    Some(DXGI_FORMAT(match format.0 {
        27 => 28,
        28 | 29 | 31 => format.0,
        87 | 91 => format.0,
        90 => 87,
        23 => 24,
        24 => 24,
        9 => 10,
        10 | 11 | 13 => format.0,
        _ => return None,
    }))
}

/// The back buffer view a plane of `format` is drawn through: sRGB when
/// the plane is (its texels are decoded when read and encoded again here,
/// so its bytes arrive as stored).
fn target_format(plane: DXGI_FORMAT, back: DXGI_FORMAT) -> DXGI_FORMAT {
    match (plane.0, back) {
        (29, DXGI_FORMAT_R8G8B8A8_UNORM) => DXGI_FORMAT_R8G8B8A8_UNORM_SRGB,
        (91, DXGI_FORMAT_B8G8R8A8_UNORM) => DXGI_FORMAT_B8G8R8A8_UNORM_SRGB,
        _ => back,
    }
}

/// A plane's source rectangle as fractions of its texture, `[u0, v0, u1,
/// v1]`: the whole texture unless the title's rectangle lies inside it.
fn source_fractions(src: Option<[i32; 4]>, width: u32, height: u32) -> [f32; 4] {
    match src {
        Some([l, t, r, b])
            if 0 <= l && l < r && r as u32 <= width && 0 <= t && t < b && b as u32 <= height =>
        {
            let (w, h) = (width as f32, height as f32);
            [l as f32 / w, t as f32 / h, r as f32 / w, b as f32 / h]
        }
        _ => [0.0, 0.0, 1.0, 1.0],
    }
}

/// A plane's placement in pixels of an output of `size`, `[left, top,
/// right, bottom]`: the whole output unless the title's fractions are a
/// rectangle inside 0..1.
fn placement(dest: Option<[f32; 4]>, (width, height): (u32, u32)) -> [f32; 4] {
    let (w, h) = (width as f32, height as f32);
    match dest {
        Some([l, t, r, b])
            if [l, t, r, b].iter().all(|v| v.is_finite())
                && 0.0 <= l
                && l < r
                && r <= 1.0
                && 0.0 <= t
                && t < b
                && b <= 1.0 =>
        {
            [l * w, t * h, r * w, b * h]
        }
        _ => [0.0, 0.0, w, h],
    }
}

struct Layer<'a> {
    plane: &'a PresentPlane,
    desc: D3D12_RESOURCE_DESC,
    view: DXGI_FORMAT,
    target: DXGI_FORMAT,
    pipeline: ID3D12PipelineState,
}

pub(super) struct Composition<'a> {
    pass: &'static ComposePass,
    layers: Vec<Layer<'a>>,
    views: ID3D12DescriptorHeap,
    pub(super) format: DXGI_FORMAT,
    pub(super) size: (u32, u32),
}

impl<'a> Composition<'a> {
    pub(super) fn new(device: &ID3D12Device, planes: &'a [PresentPlane], premultiplied: bool) -> Option<Self> {
        let descriptions: Vec<_> = planes.iter().map(|p| {
            // SAFETY: the plane owns a live host texture.
            unsafe { p.texture.texture.GetDesc() }
        }).collect();
        static LAST: Mutex<Vec<(u64, u32, i32)>> = Mutex::new(Vec::new());
        let signature: Vec<_> = descriptions.iter().map(|d| (d.Width, d.Height, d.Format.0)).collect();
        let mut last = LAST.lock().unwrap_or_else(|e| e.into_inner());
        if *last != signature {
            let mut text = format!("{} planes, stride 72:", planes.len());
            for (width, height, format) in &signature {
                text += &format!(" [{width}x{height} f{format}]");
            }
            tracing::info!(planes = %text, "present planes changed");
            *last = signature;
        }
        drop(last);

        let largest = descriptions.iter().max_by_key(|d| d.Width * u64::from(d.Height))?;
        let format = swap_format(largest.Format)?;
        let size = (largest.Width as u32, largest.Height);
        let pass = compose_pass(device)?;
        let layers = planes.iter().zip(descriptions).enumerate().map(|(i, (plane, desc))| {
            if desc.Flags.0 & D3D12_RESOURCE_FLAG_DENY_SHADER_RESOURCE.0 != 0
                || desc.SampleDesc.Count != 1
                || desc.Dimension != D3D12_RESOURCE_DIMENSION_TEXTURE2D
                || desc.DepthOrArraySize != 1
            {
                return None;
            }
            let view = view_format(desc.Format)?;
            let target = target_format(desc.Format, format);
            let blend = if i == 0 { 0 } else if premultiplied { 2 } else { 1 };
            let pipeline = pass.pipeline(device, target, blend)?;
            Some(Layer { plane, desc, view, target, pipeline })
        }).collect::<Option<Vec<_>>>()?;
        // SAFETY: creation from a plain descriptor. Each recording owns
        // its heap so an in-flight shader never reads overwritten views.
        let views = unsafe {
            device.CreateDescriptorHeap(&D3D12_DESCRIPTOR_HEAP_DESC {
                Type: D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV,
                NumDescriptors: layers.len() as u32,
                Flags: D3D12_DESCRIPTOR_HEAP_FLAG_SHADER_VISIBLE,
                NodeMask: 0,
            })
        }.inspect_err(|e| tracing::warn!(error = %e, "no descriptor heap for present planes")).ok()?;
        Some(Self { pass, layers, views, format, size })
    }

    /// Records under the submission lock, so the states are those of the
    /// preceding GPU work even when translation and submission are deferred.
    /// The source barriers are paired and the tracked states stay valid.
    pub(super) fn record(&self, host: &Host, copy: &HostList, back: &ID3D12Resource) {
        const READ: D3D12_RESOURCE_STATES = D3D12_RESOURCE_STATE_PIXEL_SHADER_RESOURCE;
        let list = &copy.list;
        let mut before = vec![transition(back, D3D12_RESOURCE_STATE_PRESENT, D3D12_RESOURCE_STATE_RENDER_TARGET)];
        let mut after = vec![transition(back, D3D12_RESOURCE_STATE_RENDER_TARGET, D3D12_RESOURCE_STATE_COPY_DEST)];
        let mut seen = std::collections::HashSet::new();
        for layer in &self.layers {
            let texture = &layer.plane.texture;
            let state = D3D12_RESOURCE_STATES(texture.state.load(Ordering::Relaxed) as i32);
            if state != READ && seen.insert(texture.texture.as_raw()) {
                before.push(transition(&texture.texture, state, READ));
                after.push(transition(&texture.texture, READ, state));
            }
        }
        // Hold both the D3D heap and the host allocations until this list
        // is reset after its flight. The swap chain holds back until resize,
        // which settles all flights before replacing its buffers.
        copy.keep(&self.views);
        copy.deferred().present_planes.extend(self.layers.iter().map(|l| l.plane.texture.clone()));

        // SAFETY: list is open, views are private to the recording, and the
        // source textures and back buffer are live through execution.
        unsafe {
            let step = host.device.GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV);
            let cpu = self.views.GetCPUDescriptorHandleForHeapStart();
            let gpu = self.views.GetGPUDescriptorHandleForHeapStart();
            list.ResourceBarrier(&before);
            let clear = host.rtv(Some(back), None);
            list.ClearRenderTargetView(clear, &[0.0, 0.0, 0.0, 1.0], None);
            list.SetDescriptorHeaps(&[Some(self.views.clone())]);
            list.SetGraphicsRootSignature(&self.pass.root);
            list.IASetPrimitiveTopology(windows::Win32::Graphics::Direct3D::D3D_PRIMITIVE_TOPOLOGY(4));
            for (i, layer) in self.layers.iter().enumerate() {
                let offset = i * step as usize;
                let view = D3D12_SHADER_RESOURCE_VIEW_DESC {
                    Format: layer.view,
                    ViewDimension: D3D12_SRV_DIMENSION_TEXTURE2D,
                    Shader4ComponentMapping: D3D12_DEFAULT_SHADER_4_COMPONENT_MAPPING,
                    Anonymous: D3D12_SHADER_RESOURCE_VIEW_DESC_0 {
                        Texture2D: D3D12_TEX2D_SRV { MipLevels: 1, ..Default::default() },
                    },
                };
                host.device.CreateShaderResourceView(
                    &layer.plane.texture.texture, Some(&view),
                    D3D12_CPU_DESCRIPTOR_HANDLE { ptr: cpu.ptr + offset },
                );
                let rtv = host.rtv(Some(back), Some(&D3D12_RENDER_TARGET_VIEW_DESC {
                    Format: layer.target,
                    ViewDimension: D3D12_RTV_DIMENSION_TEXTURE2D,
                    ..Default::default()
                }));
                let source = source_fractions(layer.plane.src, layer.desc.Width as u32, layer.desc.Height);
                let [l, t, r, b] = placement(layer.plane.dest, self.size);
                list.SetPipelineState(&layer.pipeline);
                list.SetGraphicsRootDescriptorTable(0, D3D12_GPU_DESCRIPTOR_HANDLE { ptr: gpu.ptr + offset as u64 });
                list.SetGraphicsRoot32BitConstants(1, 4, source.as_ptr().cast(), 0);
                list.OMSetRenderTargets(1, Some(&rtv), false, None);
                list.RSSetViewports(&[D3D12_VIEWPORT {
                    TopLeftX: l, TopLeftY: t, Width: r - l, Height: b - t, MinDepth: 0.0, MaxDepth: 1.0,
                }]);
                list.RSSetScissorRects(&[RECT {
                    left: l as i32, top: t as i32, right: r.ceil() as i32, bottom: b.ceil() as i32,
                }]);
                list.DrawInstanced(3, 1, 0, 0);
            }
            list.ResourceBarrier(&after);
        }
    }
}
