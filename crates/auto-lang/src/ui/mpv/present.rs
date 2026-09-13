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
                entry_point: Some("fs_main"),
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

/// WGSL：全屏三角形 + 采样。alpha 强制 1.0（理由见模块文档）。
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

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let c = textureSample(frame_tex, frame_sampler, in.uv);
    // mpv 的 "rgb0" 第 4 字节是未初始化垃圾，绝不能当 alpha 用（否则画面随机
    // 变半透明 = 闪烁）。这里只取 rgb，alpha 恒为不透明。
    return vec4<f32>(c.rgb, 1.0);
}
"#;
