use playerone_sdk::Camera;
use playerone_sdk::ImageFormat;

pub fn main() {
    let camera_description = Camera::all_cameras()
        .into_iter()
        .nth(0)
        .expect("No cameras found");

    let mut camera = camera_description.open().expect("opening camera");

    println!("camera properties:\n{:#?}\n", camera.properties());

    let bounds = camera.config_bounds();
    println!("camera bounds:\n{:#?}\n", bounds);

    camera
        .set_image_format(ImageFormat::RAW8)
        .expect("setting image format");

    camera.set_exposure(8000, true).expect("setting exposure");
    camera.set_gain(200, true).expect("setting gain");

    camera
        .set_auto_target_brightness(5)
        .expect("setting auto target brightness");

    camera
        .set_image_size(
            camera.properties().max_width,
            camera.properties().max_height,
        )
        .expect("setting image size");

    if camera.properties().is_support_hard_bin {
        camera.set_hardware_bin(true).expect("setting hardware bin");
    }

    // this changes the image size
    camera.set_bin(2).expect("setting bin");

    eprintln!("camera image size: {:?}", camera.image_size());

    let (camera_w, camera_h) = camera.image_size();

    let (tx, rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        camera
            .stream(Some(1000), move |_camera, buffer| {
                let mut v = Vec::with_capacity(buffer.len());
                v.extend_from_slice(&buffer);
                tx.send(v).unwrap();
                true
            })
            .expect("stream failed");
    });

    winit_display::run_window(camera_w, camera_h, rx);
}

mod winit_display {
    use std::sync::Arc;
    use std::sync::mpsc::Receiver;
    use std::time::Instant;

    use wgpu::{
        Features, InstanceDescriptor, MemoryHints, SurfaceConfiguration, TextureViewDescriptor,
    };
    use winit::application::ApplicationHandler;
    use winit::event::WindowEvent;
    use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
    use winit::window::{Window, WindowId};

    pub fn run_window(camera_w: u32, camera_h: u32, camera_stream: Receiver<Vec<u8>>) {
        let event_loop = EventLoop::new().unwrap();

        #[allow(deprecated)]
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );

        let instance = wgpu::Instance::new(InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            flags: Default::default(),
            dx12_shader_compiler: Default::default(),
            gles_minor_version: Default::default(),
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }))
        .expect("no adapter found");

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("device"),
                required_features: Features::default(),
                required_limits: Default::default(),
                memory_hints: MemoryHints::Performance,
            },
            None,
        ))
        .expect("no device found");

        let queue = Arc::new(queue);

        let camera_texture = Arc::new(device.create_texture(&wgpu::TextureDescriptor {
            label: Some("camera_texture"),
            size: wgpu::Extent3d {
                width: camera_w,
                height: camera_h,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: Default::default(),
        }));

        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8Unorm,
            width: 800,
            height: 600,
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 1,
            alpha_mode: Default::default(),
            view_formats: vec![
                wgpu::TextureFormat::Bgra8UnormSrgb,
                wgpu::TextureFormat::Bgra8Unorm,
            ],
        };

        surface.configure(&device, &config);

        let blit_shader = r#"
@group(0) @binding(0) var src: texture_2d<f32>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) frag_uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) VertexIndex: u32) -> VertexOutput {
    var pos = vec2(0.0, 0.0);
    switch VertexIndex {
        case 0u: {pos = vec2(-1.0, -1.0);}
        case 1u: {pos = vec2(3.0, -1.0);}
        case 2u: {pos = vec2(-1.0, 3.0);}
        default: {}
    }

    let uv = vec2(pos.x * 0.5 + 0.5, 0.5 - pos.y * 0.5);
    return VertexOutput(vec4(pos, 0.0, 1.0), uv);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let dims = textureDimensions(src);
    let pos = vec2<u32>(in.frag_uv * vec2<f32>(f32(dims.x), f32(dims.y)));
    var v = textureLoad(src, pos, 0).r;
    v = v * v;
 
    return vec4(v, v, v, 1.0);
}
        "#;

        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("blit_shader"),
            source: wgpu::ShaderSource::Wgsl(blit_shader.into()),
        });

        let bg_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bg_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline_layout"),
            bind_group_layouts: &[&bg_layout],
            push_constant_ranges: &[],
        });

        let blit_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("blit_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format.add_srgb_suffix(),
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            cache: None,
        });

        let blit_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bg"),
            layout: &bg_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(
                    &camera_texture.create_view(&Default::default()),
                ),
            }],
        });

        std::thread::spawn({
            let queue = Arc::clone(&queue);
            let camera_texture = Arc::clone(&camera_texture);
            move || {
                for pixels in camera_stream {
                    queue.write_texture(
                        camera_texture.as_image_copy(),
                        &pixels,
                        wgpu::ImageDataLayout {
                            offset: 0,
                            bytes_per_row: Some(camera_texture.width()),
                            rows_per_image: Some(camera_texture.height()),
                        },
                        camera_texture.size(),
                    );
                    //queue.submit([]);
                }
            }
        });

        struct App {
            window: Arc<Window>,
            close_requested: bool,
            last_v: Option<Instant>,
            device: wgpu::Device,
            queue: Arc<wgpu::Queue>,
            surface: wgpu::Surface<'static>,
            config: SurfaceConfiguration,

            blit_pipeline: wgpu::RenderPipeline,
            blit_bg: wgpu::BindGroup,

            i: usize,
        }

        impl ApplicationHandler for App {
            fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

            fn window_event(&mut self, _: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
                match event {
                    WindowEvent::CloseRequested => {
                        self.close_requested = true;
                    }
                    WindowEvent::Resized(new_size) => {
                        let (width, height) = {
                            let size = new_size;
                            (size.width, size.height)
                        };
                        self.config.width = width;
                        self.config.height = height;

                        self.surface.configure(&self.device, &self.config);
                    }
                    WindowEvent::RedrawRequested => {
                        if let Some(ref mut last) = self.last_v {
                            self.i += 1;
                            if self.i == 100 {
                                self.i = 0;
                                let fps = 100.0 / last.elapsed().as_secs_f64();
                                *last = Instant::now();
                                println!("FPS: {:.2}", fps);
                            }
                        } else {
                            self.last_v = Some(Instant::now());
                        }

                        let frame = self.surface.get_current_texture().unwrap();

                        let mut encoder =
                            self.device
                                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                    label: Some("encoder"),
                                });

                        {
                            let mut rpass =
                                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                    label: Some("render_pass"),
                                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                        view: &frame.texture.create_view(&TextureViewDescriptor {
                                            label: None,
                                            format: Some(self.config.format.add_srgb_suffix()),
                                            dimension: None,
                                            aspect: Default::default(),
                                            base_mip_level: 0,
                                            mip_level_count: None,
                                            base_array_layer: 0,
                                            array_layer_count: None,
                                        }),
                                        resolve_target: None,
                                        ops: wgpu::Operations {
                                            load: wgpu::LoadOp::Load,
                                            store: wgpu::StoreOp::Store,
                                        },
                                    })],
                                    depth_stencil_attachment: None,
                                    timestamp_writes: None,
                                    occlusion_query_set: None,
                                });

                            rpass.set_pipeline(&self.blit_pipeline);
                            rpass.set_bind_group(0, &self.blit_bg, &[]);
                            rpass.draw(0..3, 0..1);
                        }

                        self.queue.submit(Some(encoder.finish()));

                        self.window.pre_present_notify();
                        frame.present();
                    }
                    _ => {}
                }
            }

            fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
                if !self.close_requested {
                    self.window.request_redraw();
                }

                event_loop.set_control_flow(ControlFlow::Poll);

                if self.close_requested {
                    event_loop.exit();
                }
            }
        }

        event_loop
            .run_app(&mut App {
                window: window.clone(),
                close_requested: false,
                last_v: None,
                device,
                queue,
                surface,
                config,
                blit_pipeline,
                blit_bg,
                i: 0,
            })
            .unwrap();
    }
}
