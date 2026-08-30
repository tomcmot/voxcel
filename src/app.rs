use anyhow::Result;
use glam::{Mat4, Vec3};
use sdl3::{
    gpu::{
        BlendFactor, BlendOp, Buffer, BufferBinding, BufferUsageFlags, ColorTargetBlendState, ColorTargetDescription, CommandBuffer, CompareOp, CopyPass, DepthStencilState, Device, Filter, GraphicsPipeline, GraphicsPipelineTargetInfo, IndexElementSize, PrimitiveType, RenderPass, SampleCount, Sampler, SamplerAddressMode, SamplerCreateInfo, SamplerMipmapMode, Texture, TextureCreateInfo, TextureFormat, TextureRegion, TextureSamplerBinding, TextureTransferInfo, TextureType, TextureUsage, TransferBufferUsage, VertexInputState,
    }, video::Window,
};

use crate::{camera::Camera, chunk_render::Vertex, shaders::{FRAG_SHADER, VERTEX_SHADER, create_shaders}};
use crate::{
    gpu_mem::upload_data,
    world::{World},
};

const TEXTURE_PATH: &'static str = "assets/blocks.png";
const MASK_PATH: &'static str = "assets/block_masks.png";
const SHADING_PATH: &'static str = "assets/light.png";

const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

#[repr(C)]
#[derive(Debug)]
struct CameraBuffer {
    proj_view: Mat4,
    model: Mat4,
}

#[repr(C)]
struct LightingBuffer {
    ambient_color: Vec3,
    _pad0: f32,
    light_dir: Vec3,
    _pad1: f32,
    light_color: Vec3,
    _pad2: f32,
}

impl LightingBuffer {
    pub fn new(ambient_color: Vec3, light_dir: Vec3, light_color: Vec3) -> Self {
        LightingBuffer {
            ambient_color,
            _pad0: 0.,
            light_dir,
            _pad1: 0.,
            light_color,
            _pad2: 0.,
        }
    }
}

pub struct App {
    main: GraphicsPipeline,
    pub camera: Camera,
    light: LightingBuffer,
    world: World,
    index_buffer: Buffer,
    sampler: Sampler,
    block_texture: Texture<'static>,
    light_ramp: Texture<'static>,
    mask_texture: Texture<'static>,
}

impl App {
    pub fn new(window: &Window, device: &Device, assets_path: String) -> Result<Self> {
        let (vertex_shader, frag_shader) = create_shaders(
            device,
            assets_path.clone() + "main.spv",
            VERTEX_SHADER,
            FRAG_SHADER,
        )?;
        let color_target = [ColorTargetDescription::default()
            .with_blend_state(
                ColorTargetBlendState::new()
                    .with_enable_blend(true)
                    .with_color_blend_op(BlendOp::Add)
                    .with_alpha_blend_op(BlendOp::Add)
                    .with_src_color_blendfactor(BlendFactor::SrcAlpha)
                    .with_dst_color_blendfactor(BlendFactor::OneMinusSrcAlpha)
                    .with_src_alpha_blendfactor(BlendFactor::SrcAlpha)
                    .with_dst_alpha_blendfactor(BlendFactor::OneMinusSrcAlpha),
            )
            .with_format(device.get_swapchain_texture_format(window))];
        let target_info = GraphicsPipelineTargetInfo::new()
            .with_color_target_descriptions(&color_target)
            .with_has_depth_stencil_target(true)
            .with_depth_stencil_format(TextureFormat::D32Float);
        let main = device
            .create_graphics_pipeline()
            .with_vertex_shader(&vertex_shader)
            .with_fragment_shader(&frag_shader)
            .with_primitive_type(PrimitiveType::TriangleList)
            .with_vertex_input_state(
                VertexInputState::default()
                    .with_vertex_buffer_descriptions(&[Vertex::buffer_desc()])
                    .with_vertex_attributes(Vertex::attributes().as_slice()),
            )
            .with_target_info(target_info)
            .with_depth_stencil_state(
                DepthStencilState::default()
                    .with_enable_depth_test(true)
                    .with_enable_depth_write(true)
                    .with_compare_op(CompareOp::Less)
                    .with_enable_stencil_test(false),
            )
            .build()?;

        let indices = World::worst_case_indexes();
        let index_buffer = device
            .create_buffer()
            .with_size((indices.len() * size_of::<u16>()) as u32)
            .with_usage(BufferUsageFlags::INDEX)
            .build()?;
        let sampler = create_sampler(&device)?;
        let block_texture = create_texture(&device, TextureType::_2DArray, 32, 32, 6)?;
        let mask_texture = create_texture(&device, TextureType::_2DArray, 32, 32, 6)?;
        let light_ramp = create_texture(&device, TextureType::_2D, 1, 32, 1)?;
        {
            let copy_commands = device.acquire_command_buffer()?;
            let copy_pass = device.begin_copy_pass(&copy_commands)?;
            upload_data(&device, &copy_pass, &index_buffer, &indices)?;
            upload_texture(&device, &copy_pass, TEXTURE_PATH, &block_texture, 6)?;
            upload_texture(&device, &copy_pass, MASK_PATH, &mask_texture, 6)?;
            upload_texture(&device, &copy_pass, SHADING_PATH, &light_ramp, 1)?;
            device.end_copy_pass(copy_pass);
            let _ = copy_commands.submit()?;
        };

        let camera = Camera::default();

        let light = LightingBuffer::new(
            Vec3::new(0.25, 0.25, 0.5),
            Vec3::new(0.3, 0.8, 0.6).normalize(),
            Vec3::new(0.8, 0.8, 0.6),
        );
        let mut world = World::new(0, 32.);
        world.queue_chunks(16)?;
        Ok(App {
            main,
            world,
            camera,
            light,
            sampler,
            block_texture,
            light_ramp,
            index_buffer,
            mask_texture,
        })
    }

    pub fn generate_world(&mut self, device: &Device, copy_pass: &CopyPass) -> Result<()> {
        self.world.load_chunks();
        self.world.generate(device, copy_pass)
    }
    pub fn render(
        &self,
        cmdbuffer: &CommandBuffer,
        render_pass: &RenderPass,
    ) {
        let sampler_bindings = [
            TextureSamplerBinding::default()
                .with_sampler(&self.sampler)
                .with_texture(&self.block_texture),
            TextureSamplerBinding::default()
                .with_sampler(&self.sampler)
                .with_texture(&self.light_ramp),
            TextureSamplerBinding::default()
                .with_sampler(&self.sampler)
                .with_texture(&self.mask_texture)
        ];

        let index_binding = BufferBinding::default()
            .with_buffer(&self.index_buffer)
            .with_offset(0);

        let camera_buffer = CameraBuffer {
            proj_view: self.camera.projection(WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32) * self.camera.view(),
            model: Mat4::IDENTITY,
        };
        render_pass.bind_graphics_pipeline(&self.main);
        cmdbuffer.push_vertex_uniform_data(0, &camera_buffer);
        cmdbuffer.push_fragment_uniform_data(0, &self.light);
        render_pass.bind_index_buffer(&index_binding, IndexElementSize::_16BIT);
        render_pass.bind_fragment_samplers(0, &sampler_bindings);
        self.world.render(cmdbuffer, render_pass);
    }
}

fn upload_texture(
    device: &Device,
    copy_pass: &CopyPass,
    path: &str,
    texture: &Texture<'static>,
    layers: u32,
) -> Result<()> {
    let img = image::open(path)?;
    let height = img.height() / layers;
    let width = img.width();
    let bytes = img.to_rgba8();
    let transfer_buffer = device
        .create_transfer_buffer()
        .with_size(width * img.height() * 4 * size_of::<u8>() as u32)
        .with_usage(TransferBufferUsage::UPLOAD)
        .build()?;
    let mut memmap = transfer_buffer.map(device, false);
    memmap.mem_mut().copy_from_slice(bytes.as_raw());
    memmap.unmap();
    for i in 0..layers {
        copy_pass.upload_to_gpu_texture(
            TextureTransferInfo::new()
                .with_transfer_buffer(&transfer_buffer)
                .with_offset(i * width * height * 4 * size_of::<u8>() as u32),
            TextureRegion::new()
                .with_texture(texture)
                .with_depth(1)
                .with_height(height)
                .with_width(width)
                .with_layer(i)
                .with_mip_level(0),
            false,
        );
    }
    Ok(())
}

fn create_sampler(device: &Device) -> Result<Sampler> {
    let sampler_info = SamplerCreateInfo::default()
        .with_min_filter(Filter::Nearest)
        .with_mag_filter(Filter::Nearest)
        .with_mipmap_mode(SamplerMipmapMode::Nearest)
        .with_address_mode_u(SamplerAddressMode::ClampToEdge)
        .with_address_mode_v(SamplerAddressMode::ClampToEdge)
        .with_address_mode_w(SamplerAddressMode::ClampToEdge)
        .with_mip_lod_bias(0.0)
        .with_min_lod(0.0)
        .with_max_lod(0.0)
        .with_enable_anisotropy(false)
        .with_enable_compare(false);
    Ok(device.create_sampler(sampler_info)?)
}

fn create_texture(
    device: &Device,
    texture_type: TextureType,
    height: u32,
    width: u32,
    layers: u32,
) -> Result<Texture<'static>> {
    let texture_info = TextureCreateInfo::default()
        .with_type(texture_type)
        .with_format(TextureFormat::R8g8b8a8Unorm)
        .with_usage(TextureUsage::SAMPLER)
        .with_height(height)
        .with_width(width)
        .with_layer_count_or_depth(layers)
        .with_num_levels(1)
        .with_sample_count(SampleCount::NoMultiSampling);
    Ok(device.create_texture(texture_info)?)
}
