use graphics_state::{GraphicsState, Height, Width};
use lyon::geom::Box2D;
use lyon::lyon_tessellation::StrokeOptions;
use lyon::math::*;

use lyon::path::PathEvent;
use lyon::tessellation;
use lyon::tessellation::geometry_builder::*;
use lyon::tessellation::StrokeTessellator;
use lyon::tessellation::{FillOptions, FillTessellator};
use pdf::utils::read_file_bytes;
use pdf::{Pdf as PdfDocument, StreamObject};
use shared::{Color, ColorSpaceWithColor};
use lyon::algorithms::path::math::Point;
use std::io::Write;

// For create_buffer_init()
use wgpu::util::DeviceExt;

use futures::executor::block_on;
use std::num::NonZeroU32;

const PRIM_BUFFER_LEN: usize = 256;

const OUTPUT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/img/page-2.png");

#[repr(C)]
#[derive(Copy, Clone)]
struct Globals {
    resolution: [f32; 2],
    scroll_offset: [f32; 2],
    zoom: f32,
    _pad: f32,
}

unsafe impl bytemuck::Pod for Globals {}
unsafe impl bytemuck::Zeroable for Globals {}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct GpuVertex {
    position: [f32; 2],
    normal: [f32; 2],
    prim_id: u32,
}
unsafe impl bytemuck::Pod for GpuVertex {}
unsafe impl bytemuck::Zeroable for GpuVertex {}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct Primitive {
    color: [f32; 4],
    translate: [f32; 2],
    z_index: i32,
    width: f32,
    angle: f32,
    scale: f32,
    _pad1: i32,
    _pad2: i32,
}

impl Primitive {
    const DEFAULT: Self = Primitive {
        color: [0.0; 4],
        translate: [0.0; 2],
        z_index: 0,
        width: 0.0,
        angle: 0.0,
        scale: 1.0,
        _pad1: 0,
        _pad2: 0,
    };
}

fn make_color_slice(color: ColorSpaceWithColor) -> [f32; 4] {
    match color {
        ColorSpaceWithColor::DeviceCMYK(_) => unimplemented!(),
        // FIXME: annoying that the shader requires the order to be inverted...
        ColorSpaceWithColor::DeviceRGB(c) => [c.blue(), c.green(), c.red(), 1.0],
        ColorSpaceWithColor::DeviceGray(_) => unimplemented!(),
    }
}

unsafe impl bytemuck::Pod for Primitive {}
unsafe impl bytemuck::Zeroable for Primitive {}

#[repr(C)]
#[derive(Copy, Clone)]
struct BgPoint {
    point: [f32; 2],
}
unsafe impl bytemuck::Pod for BgPoint {}
unsafe impl bytemuck::Zeroable for BgPoint {}

const DEFAULT_WINDOW_WIDTH: f32 = 612.0;
const DEFAULT_WINDOW_HEIGHT: f32 = 792.0;

fn main() {
    // Number of samples for anti-aliasing
    let pdf = {
        let bytes = read_file_bytes(concat!(
            env!("CARGO_WORKSPACE_DIR"),
            "/pdfs/sample-no-xref-entries/sample-no-xref-entries.pdf"
        ));
        PdfDocument::from_bytes(&bytes).expect("could't parse PDF")
    };
    let drawing = pdf
        .document
        .get_object((11, 0))
        .expect("couldn't find the drawing instructions");

    // Set to 1 to disable
    let sample_count = 1;

    let tolerance = 0.02;

    let mut cpu_primitives = Vec::with_capacity(PRIM_BUFFER_LEN);
    for _ in 0..PRIM_BUFFER_LEN {
        cpu_primitives.push(Primitive {
            color: [0.0, 0.0, 0.0, 1.0],
            z_index: 0,
            width: 0.0,
            translate: [0.0, 0.0],
            angle: 0.0,
            ..Primitive::DEFAULT
        });
    }

    let mut running_prim_id = 0;

    let mut fill_geometry: VertexBuffers<GpuVertex, u16> = VertexBuffers::new();
    let mut stroke_geometry: VertexBuffers<GpuVertex, u16> = VertexBuffers::new();

    let mut fill_tess = FillTessellator::new();
    let mut stroke_tess = StrokeTessellator::new();

    let draw_instructions = drawing.as_stream().unwrap().get_content().unwrap();
    let width = Width::new(DEFAULT_WINDOW_WIDTH);
    let height = Height::new(DEFAULT_WINDOW_HEIGHT);
    let mut graphics_state = GraphicsState::new(width, height);
    for inst in draw_instructions {
        match inst {
            StreamObject::Text(_) => unimplemented!(),
            StreamObject::CapStyle(c) => {
                graphics_state.set_cap_style(c).unwrap();
            }
            StreamObject::MoveTo(p) => {
                graphics_state.move_to(p).unwrap();
            }
            StreamObject::LineTo(p) => {
                graphics_state.line_to(p).unwrap();
            }
            StreamObject::Rect(low_left, width, height) => {
                graphics_state.rect(low_left, width, height).unwrap();
            }
            StreamObject::Fill => {
                let color = graphics_state
                    .properties
                    .non_stroke_color
                    .get_current_color();
                cpu_primitives[running_prim_id].color = make_color_slice(color);
                let paths = graphics_state.fill().unwrap();
                fill_tess
                    .tessellate(
                        paths,
                        &FillOptions::tolerance(tolerance)
                            .with_fill_rule(tessellation::FillRule::NonZero),
                        &mut BuffersBuilder::new(
                            &mut fill_geometry,
                            WithId(running_prim_id as u32),
                        ),
                    )
                    .unwrap();
                running_prim_id += 1;
            }
            StreamObject::Stroke(close) => {
                let properties = graphics_state.properties();
                let options = StrokeOptions::tolerance(tolerance)
                    .with_line_cap(properties.line_cap)
                    .with_line_width(*properties.line_width);
                let color = graphics_state.properties.stroke_color.get_current_color();
                cpu_primitives[running_prim_id].color = make_color_slice(color);
                cpu_primitives[running_prim_id].width = (*properties.line_width) / 2.0;
                let paths = graphics_state.stroke(close).unwrap();
                stroke_tess
                    .tessellate(
                        paths,
                        &options,
                        &mut BuffersBuilder::new(
                            &mut stroke_geometry,
                            WithId(running_prim_id as u32),
                        ),
                    )
                    .unwrap();
                running_prim_id += 1;
            }
            StreamObject::LineWidth(w) => {
                graphics_state.set_line_width(w).unwrap();
            }
            StreamObject::NonStrokeColor(c) => {
                graphics_state.set_non_stroke_color(c).unwrap();
            }
            StreamObject::StrokeColor(c) => {
                graphics_state.set_stroke_color(c).unwrap();
            }
            StreamObject::StrokeColorSpace(cs) => {
                graphics_state.set_stroke_color_space(cs).unwrap();
            }
            StreamObject::NonStrokeColorSpace(cs) => {
                graphics_state.set_non_stroke_color_space(cs).unwrap();
            }
            StreamObject::DashPattern(d) => {
                graphics_state.set_dash_pattern(d).unwrap();
            }
        }
    }

    let fill_range = 0..(fill_geometry.indices.len() as u32);

    let stroke_range = 0..(stroke_geometry.indices.len() as u32);

    let mut bg_geometry: VertexBuffers<BgPoint, u16> = VertexBuffers::new();

    fill_tess
        .tessellate_rectangle(
            &Box2D::default(),
            &FillOptions::DEFAULT,
            &mut BuffersBuilder::new(&mut bg_geometry, Custom),
        )
        .unwrap();

    // // Stroke primitive
    // cpu_primitives[stroke_prim_id] = Primitive {
    //     color: [0.0, 1.0, 0.0, 1.0],
    //     // TODO: Why 5.0? Stroke width / 2?
    //     width: 5.0,
    //     ..Primitive::DEFAULT
    // };
    // // Main fill primitive
    // cpu_primitives[fill_prim_id] = Primitive {
    //     color: [0.0, 0.0, 1.0, 1.0],
    //     width: 5.0,
    //     ..Primitive::DEFAULT
    // };

    // create an instance
    let instance = wgpu::Instance::new(wgpu::Backends::all());

    // create an adapter
    let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::LowPower,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .unwrap();
    // create a device and a queue
    let (device, queue) = block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: None,
            features: wgpu::Features::default(),
            limits: wgpu::Limits::default(),
        },
        None,
    ))
    .unwrap();

    let vbo_fill = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&fill_geometry.vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let ibo_fill = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&fill_geometry.indices),
        usage: wgpu::BufferUsages::INDEX,
    });

    let vbo_stroke = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&stroke_geometry.vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let ibo_stroke = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&stroke_geometry.indices),
        usage: wgpu::BufferUsages::INDEX,
    });

    let bg_vbo = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&bg_geometry.vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let bg_ibo = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&bg_geometry.indices),
        usage: wgpu::BufferUsages::INDEX,
    });

    let prim_buffer_byte_size = (PRIM_BUFFER_LEN * std::mem::size_of::<Primitive>()) as u64;
    let globals_buffer_byte_size = std::mem::size_of::<Globals>() as u64;

    let prims_ubo = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Prims ubo"),
        size: prim_buffer_byte_size,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let globals_ubo = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Globals ubo"),
        size: globals_buffer_byte_size,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let vs_module = &device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        label: Some("Geometry vs"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/geometry.vs.wgsl").into()),
    });
    let fs_module = &device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        label: Some("Geometry fs"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/geometry.fs.wgsl").into()),
    });
    let bg_vs_module = &device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        label: Some("Background vs"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/background.vs.wgsl").into()),
    });
    let bg_fs_module = &device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        label: Some("Background fs"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/background.fs.wgsl").into()),
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Bind group layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(globals_buffer_byte_size),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(prim_buffer_byte_size),
                },
                count: None,
            },
        ],
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Bind group"),
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(globals_ubo.as_entire_buffer_binding()),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Buffer(prims_ubo.as_entire_buffer_binding()),
            },
        ],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
        label: None,
    });

    let render_pipeline_descriptor = wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: vs_module,
            entry_point: "main",
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<GpuVertex>() as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[
                    // position:
                    wgpu::VertexAttribute {
                        offset: 0,
                        format: wgpu::VertexFormat::Float32x2,
                        shader_location: 0,
                    },
                    // normal:
                    wgpu::VertexAttribute {
                        offset: 8,
                        format: wgpu::VertexFormat::Float32x2,
                        shader_location: 1,
                    },
                    // prim_id:
                    wgpu::VertexAttribute {
                        offset: 16,
                        format: wgpu::VertexFormat::Uint32,
                        shader_location: 2,
                    },
                ],
            }],
        },
        fragment: Some(wgpu::FragmentState {
            module: fs_module,
            entry_point: "main",
            targets: &[wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Bgra8Unorm,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            }],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            polygon_mode: wgpu::PolygonMode::Fill,
            front_face: wgpu::FrontFace::Ccw,
            strip_index_format: None,
            cull_mode: Some(wgpu::Face::Back),
            clamp_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: sample_count,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
    };

    let render_pipeline = device.create_render_pipeline(&render_pipeline_descriptor);

    let bg_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: bg_vs_module,
            entry_point: "main",
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<Point>() as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[wgpu::VertexAttribute {
                    offset: 0,
                    format: wgpu::VertexFormat::Float32x2,
                    shader_location: 0,
                }],
            }],
        },
        fragment: Some(wgpu::FragmentState {
            module: bg_fs_module,
            entry_point: "main",
            targets: &[wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Bgra8Unorm,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            }],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            polygon_mode: wgpu::PolygonMode::Fill,
            front_face: wgpu::FrontFace::Ccw,
            strip_index_format: None,
            cull_mode: None,
            clamp_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: sample_count,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
    });

    let mut surface_desc = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: wgpu::TextureFormat::Bgra8Unorm,
        width: DEFAULT_WINDOW_WIDTH as u32,
        height: DEFAULT_WINDOW_HEIGHT as u32,
        present_mode: wgpu::PresentMode::Mailbox,
    };

    surface_desc.width = DEFAULT_WINDOW_WIDTH as u32;
    surface_desc.height = DEFAULT_WINDOW_HEIGHT as u32;

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Encoder"),
    });

    queue.write_buffer(
        &globals_ubo,
        0,
        bytemuck::cast_slice(&[Globals {
            resolution: [DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT],
            zoom: 1.0,
            scroll_offset: [0.0, 0.0],
            _pad: 0.0,
        }]),
    );

    queue.write_buffer(&prims_ubo, 0, bytemuck::cast_slice(&cpu_primitives));

    let texture_desc = wgpu::TextureDescriptor {
        size: wgpu::Extent3d {
            width: DEFAULT_WINDOW_WIDTH as u32,
            height: DEFAULT_WINDOW_HEIGHT as u32,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Bgra8Unorm,
        usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
        label: None,
    };

    let texture = device.create_texture(&texture_desc);
    let texture_view = texture.create_view(&Default::default());

    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &texture_view,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                    store: true,
                },
                resolve_target: None,
            }],
            depth_stencil_attachment: None,
        });

        pass.set_pipeline(&render_pipeline);
        pass.set_bind_group(0, &bind_group, &[]);

        pass.set_index_buffer(ibo_fill.slice(..), wgpu::IndexFormat::Uint16);
        pass.set_vertex_buffer(0, vbo_fill.slice(..));
        // 0..1 = red fill, 1..2 = black fill, 2..3 = black
        pass.draw_indexed(fill_range, 0, 0..1);

        pass.set_index_buffer(ibo_stroke.slice(..), wgpu::IndexFormat::Uint16);
        pass.set_vertex_buffer(0, vbo_stroke.slice(..));
        // 0..1 = green fill, 1..2 = red fill, 2..3 = white/clear
        pass.draw_indexed(stroke_range, 0, 0..1);

        // Draw background
        pass.set_pipeline(&bg_pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.set_index_buffer(bg_ibo.slice(..), wgpu::IndexFormat::Uint16);
        pass.set_vertex_buffer(0, bg_vbo.slice(..));
        pass.draw_indexed(0..6, 0, 0..1);
    }

    // It is a WebGPU requirement that ImageCopyBuffer.layout.bytes_per_row %
    // wgpu::COPY_BYTES_PER_ROW_ALIGNMENT == 0 So we calculate padded_bytes_per_row
    // by rounding unpadded_bytes_per_row up to the next multiple of
    // wgpu::COPY_BYTES_PER_ROW_ALIGNMENT. https://en.wikipedia.org/wiki/Data_structure_alignment#Computing_padding
    let buffer_dimensions = BufferDimensions::new(
        DEFAULT_WINDOW_WIDTH as usize,
        DEFAULT_WINDOW_HEIGHT as usize,
    );

    // The output buffer lets us retrieve the data as an array
    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: (buffer_dimensions.padded_bytes_per_row * buffer_dimensions.height) as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    encoder.copy_texture_to_buffer(
        wgpu::ImageCopyTexture {
            aspect: wgpu::TextureAspect::All,
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
        },
        wgpu::ImageCopyBuffer {
            buffer: &output_buffer,
            layout: wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(
                    buffer_dimensions.padded_bytes_per_row as u32,
                )
                .unwrap()
                .into(),
                rows_per_image: Some(
                    NonZeroU32::new(DEFAULT_WINDOW_HEIGHT as u32)
                        .expect("oops must have been zero"),
                ),
            },
        },
        texture_desc.size,
    );

    queue.submit(Some(encoder.finish()));

    block_on(write_to_disk(
        OUTPUT,
        device,
        buffer_dimensions,
        output_buffer,
    ))
    .unwrap();
}

/// This vertex constructor forwards the positions and normals provided by the
/// tessellators and add a shape id.
pub struct WithId(pub u32);

impl FillVertexConstructor<GpuVertex> for WithId {
    fn new_vertex(&mut self, vertex: tessellation::FillVertex) -> GpuVertex {
        GpuVertex {
            position: vertex.position().to_array(),
            normal: [0.0, 0.0],
            prim_id: self.0,
        }
    }
}

impl StrokeVertexConstructor<GpuVertex> for WithId {
    fn new_vertex(&mut self, vertex: tessellation::StrokeVertex) -> GpuVertex {
        GpuVertex {
            position: vertex.position_on_path().to_array(),
            normal: vertex.normal().to_array(),
            prim_id: self.0,
        }
    }
}

pub struct Custom;

impl FillVertexConstructor<BgPoint> for Custom {
    fn new_vertex(&mut self, vertex: tessellation::FillVertex) -> BgPoint {
        BgPoint {
            point: vertex.position().to_array(),
        }
    }
}

struct BufferDimensions {
    width: usize,
    height: usize,
    unpadded_bytes_per_row: usize,
    padded_bytes_per_row: usize,
}

impl BufferDimensions {
    fn new(width: usize, height: usize) -> Self {
        let bytes_per_pixel = std::mem::size_of::<u32>();
        let unpadded_bytes_per_row = width * bytes_per_pixel;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize;
        let padded_bytes_per_row_padding = (align - unpadded_bytes_per_row % align) % align;
        let padded_bytes_per_row = unpadded_bytes_per_row + padded_bytes_per_row_padding;
        Self {
            width,
            height,
            unpadded_bytes_per_row,
            padded_bytes_per_row,
        }
    }
}

async fn write_to_disk<'a, 'b>(
    filename: &str,
    device: wgpu::Device,
    buffer_dimensions: BufferDimensions,
    output_buffer: wgpu::Buffer,
) -> Result<(), ()> {
    // Note that we're not calling `.await` here.
    let buffer_slice = output_buffer.slice(..);
    let buffer_future = buffer_slice.map_async(wgpu::MapMode::Read);
    // Poll the device in a blocking manner so that our future resolves.
    // In an actual application, `device.poll(...)` should
    // be called in an event loop or on another thread.
    device.poll(wgpu::Maintain::Wait);
    // If a file system is available, write the buffer as a PNG
    let has_file_system_available = cfg!(not(target_arch = "wasm32"));
    if !has_file_system_available {
        return Ok(());
    }

    if let Ok(()) = buffer_future.await {
        let padded_buffer = buffer_slice.get_mapped_range();

        let mut png_encoder = png::Encoder::new(
            std::fs::File::create(filename).unwrap(),
            buffer_dimensions.width as u32,
            buffer_dimensions.height as u32,
        );
        png_encoder.set_depth(png::BitDepth::Eight);
        png_encoder.set_color(png::ColorType::RGBA);
        let mut png_writer = png_encoder
            .write_header()
            .unwrap()
            .into_stream_writer_with_size(buffer_dimensions.unpadded_bytes_per_row);

        // from the padded_buffer we write just the unpadded bytes into the image
        for chunk in padded_buffer.chunks(buffer_dimensions.padded_bytes_per_row) {
            png_writer
                .write_all(&chunk[..buffer_dimensions.unpadded_bytes_per_row])
                .unwrap();
        }
        png_writer.finish().unwrap();

        // With the current interface, we have to make sure all mapped views are
        // dropped before we unmap the buffer.
        drop(padded_buffer);

        output_buffer.unmap();
    }

    Ok(())
}
