//! 把 [`VideoFrameChannel`] 的持久纹理**画到目标上**（T-17 的「上屏」那一半）。
//!
//! 这里是一个最小的 wgpu 全屏三角形采样管线，形状刻意做成 T-19 可直接搬进
//! iced 自定义 shader widget 的 `Pipeline`/`Primitive` 的样子
//! （`iced_widget::shader::Program` 的 `Pipeline::new(device, queue, format)` 恰好
//! 交出 `&wgpu::Device`/`&wgpu::Queue`，`Primitive::render` 交出
//! `&mut CommandEncoder` 与 `&TextureView`）——**所以本模块不依赖 iced 的任何 widget 类型**，
//! 只依赖 wgpu。
//!
//! # shader 里那个 `1.0` 不是随手写的
//!
//! mpv 的 SW 输出格式是 `"rgb0"`：第 4 个字节是 **未初始化的垃圾**
//! （`render.h` 原文 "the '0' component contains uninitialized garbage"）。
//! 如果把它当 alpha 用，画面会随机变成半透明甚至整帧「消失」——**那就是闪烁**。
//! 故片元着色器显式输出 `vec4(rgb, 1.0)`，不读纹理的 alpha。
//!
//! # 采样与方向
//!
//! SW 后端不支持 `MPV_RENDER_PARAM_FLIP_Y`（`render.h:156` 记为 unsupported），
//! 所以第 0 行就是画面顶部；上传到纹理后**不做翻转**，采样时以 v=0 对应顶行。

use iced_wgpu::wgpu;

/// 建一个**无 surface** 的 headless wgpu 设备（测试、spike 探针、离屏渲染用）。
///
/// 不需要显示器/窗口，因此可以在 CI 与无头环境里跑通道的实测。
pub fn headless_device() -> Result<(wgpu::Device, wgpu::Queue), String> {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
    let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: None,
    }))
    .map_err(|e| format!("request_adapter: {e:?}"))?;
    let info = adapter.get_info();
    log::debug!(
        "mpv 离屏 wgpu 适配器：{} ({:?}, {:?})",
        info.name,
        info.backend,
        info.device_type
    );
    let (device, queue) = block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("mpv headless"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::downlevel_defaults().using_resolution(adapter.limits()),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
        ..Default::default()
    }))
    .map_err(|e| format!("request_device: {e:?}"))?;
    Ok((device, queue))
}

/// 极简 executor：不想为此引入 pollster/futures 依赖（本模块只在建设备时用一次）。
fn block_on<F: std::future::Future>(f: F) -> F::Output {
    use std::task::{Context, Poll, Wake, Waker};
    struct Noop;
    impl Wake for Noop {
        fn wake(self: std::sync::Arc<Self>) {}
    }
    let waker = Waker::from(std::sync::Arc::new(Noop));
    let mut cx = Context::from_waker(&waker);
    let mut f = std::pin::pin!(f);
    loop {
        match f.as_mut().poll(&mut cx) {
            Poll::Ready(v) => return v,
            Poll::Pending => std::thread::yield_now(),
        }
    }
}

/// 采样视频纹理的全屏 blit 管线。
pub struct VideoPresenter {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    bind_group: Option<wgpu::BindGroup>,
    bound_view: Option<*const wgpu::TextureView>,
}

impl VideoPresenter {
    /// 建管线（目标格式 = 交换链/离屏目标的格式）。
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("video blit"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(WGSL)),
        });

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("video blit layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("video blit pipeline layout"),
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("video blit pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                // PLAN-712 T-03：按目标格式选编码臂——sRGB 目标硬件编码，写
                // 显示线性（`fs_main`）；非 sRGB 目标（本构建恒此，见下）须在
                // shader 内完成 sRGB 编码（`fs_main_raw_target`）。
                entry_point: Some(if format.is_srgb() {
                    "fs_main"
                } else {
                    "fs_main_raw_target"
                }),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // 线性过滤：视频缩放时比最近邻观感好；边界用 ClampToEdge 避免边缘渗色。
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("video blit sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        Self {
            pipeline,
            layout,
            sampler,
            bind_group: None,
            bound_view: None,
        }
    }

    /// 绑定纹理视图；视图指针未变时**复用**已有的 bind group。
    ///
    /// 复用是有意的：每帧重建 bind group 会把「帧」变成分配热点，
    /// 而通道本身的立场就是**稳态零分配**。
    fn ensure_bound(&mut self, device: &wgpu::Device, view: &wgpu::TextureView) {
        let ptr = view as *const wgpu::TextureView;
        if self.bound_view == Some(ptr) {
            return;
        }
        self.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("video blit bind group"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        }));
        self.bound_view = Some(ptr);
    }

    /// 交出管线三件套（render pipeline / bind group layout / sampler）。
    ///
    /// 给 T-19 的 iced `Primitive::Pipeline` 用：那边要按 widget 自建 bind group
    /// （纹理是每 widget 一张的持久纹理），而管线与采样器是**全局共享**的。
    pub fn into_parts(
        self,
    ) -> (
        wgpu::RenderPipeline,
        wgpu::BindGroupLayout,
        wgpu::Sampler,
    ) {
        (self.pipeline, self.layout, self.sampler)
    }

    /// 把 `view` 画满 `target`（清屏 + 全屏三角形）。
    pub fn draw(&mut self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView, target: &wgpu::TextureView) {
        self.ensure_bound(device, view);
        let bg = self.bind_group.as_ref().expect("bind group 刚建立");
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("video blit pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, bg, &[]);
        pass.draw(0..3, 0..1);
    }
}

/// WGSL：全屏三角形 + 采样 + 传递函数归一。alpha 强制 1.0（理由见模块文档）。
///
/// # PLAN-712 T-03：BT.1886 → sRGB 传递函数归一（含目标格式分支）
///
/// 输入字节域：mpv SW rgb0 = **源签名传递函数**编码（SDR = BT.1886 ≈ γ2.4；
/// DP-3 探针：SW 路径不吃任何色彩协商）。纹理是 sRGB：硬件采样按 ~γ2.2
/// 线性化——旧 shader 直通在非 sRGB 目标上构成**双重解码**（decode∘decode
/// ≈ γ4.4+），近白内容 G/B 被压、整体偏暗偏暖——这正是 E-3 实录的机理
///（数值复算 (255,244,238)→(255,234,222) 与实录 (251,237,230) 同形）。
///
/// 本仓构建形态（T-01 实机修正）：iced 0.14 **default 含 `web-colors`** ⇒
/// `GAMMA_CORRECTION=false` ⇒ 目标恒为**非 sRGB**（Bgra8Unorm，iced 自身
/// chrome 即以 sRGB 编码字节直写）。因此：
/// * 非 sRGB 目标（`fs_main_raw_target`，本构建恒此）：shader 内完成
///   字节域 2.4→sRGB 转换后直写——与 Chromium `<video>` 字节域对齐；
/// * sRGB 目标（`fs_main`，防御性保留）：写显示线性，交硬件编码。
///
/// 两臂的公共前段：采样值 c（≈2.2 解码域）→ 还原 mpv 字节域 b =
/// sRGB_encode(c) → 按 2.4 解出真实显示线性 lin = b^2.4。白/黑点不动
///（letterbox 恒纯黑），中间调与 Web 端对齐。
const WGSL: &str = r#"
struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> VsOut {
    // 覆盖全屏的大三角形：(-1,-1) (3,-1) (-1,3)
    var xy = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 3.0, -1.0),
        vec2<f32>(-1.0,  3.0),
    );
    var out: VsOut;
    let p = xy[idx];
    out.pos = vec4<f32>(p, 0.0, 1.0);
    // 设备坐标 y 向下、纹理 v 向下，故 v = (1 - y) / 2，即第 0 行在最上方。
    out.uv = vec2<f32>((p.x + 1.0) * 0.5, (1.0 - p.y) * 0.5);
    return out;
}

@group(0) @binding(0) var frame_tex: texture_2d<f32>;
@group(0) @binding(1) var frame_sampler: sampler;

fn srgb_encode(v: vec3<f32>) -> vec3<f32> {
    let c = clamp(v, vec3<f32>(0.0), vec3<f32>(1.0));
    let hi = vec3<f32>(1.055) * pow(c, vec3<f32>(1.0 / 2.4)) - vec3<f32>(0.055);
    let lo = c * vec3<f32>(12.92);
    return select(lo, hi, c > vec3<f32>(0.0031308));
}

// 公共前段：采样 → mpv 字节域 b → 真实显示线性 lin。
fn to_display_linear(c: vec4<f32>) -> vec3<f32> {
    let b = srgb_encode(max(c.rgb, vec3<f32>(0.0)));
    return pow(b, vec3<f32>(2.4));
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let c = textureSample(frame_tex, frame_sampler, in.uv);
    // sRGB 目标：硬件按 sRGB 编码，写显示线性即可。
    return vec4<f32>(to_display_linear(c), 1.0);
}

@fragment
fn fs_main_raw_target(in: VsOut) -> @location(0) vec4<f32> {
    let c = textureSample(frame_tex, frame_sampler, in.uv);
    // 非 sRGB 目标：字节被 OS 按 sRGB 解释，须在 shader 内编码到位。
    return vec4<f32>(srgb_encode(to_display_linear(c)), 1.0);
}
"#;
