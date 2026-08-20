
use anyhow::Result;
use sdl3::gpu::{BufferRegion, CopyPass, Device, TransferBufferLocation, TransferBufferUsage};

pub fn upload_data<T>(
    device: &Device,
    copy_pass: &CopyPass,
    vertex_buffer: &sdl3::gpu::Buffer,
    data: &[T],
) -> Result<()>
where
    T: Copy,
{
    let size = (size_of::<T>() * data.len()) as u32;
    let transfer_buffer = device
        .create_transfer_buffer()
        .with_size(size)
        .with_usage(TransferBufferUsage::UPLOAD)
        .build()?;
    let mut mem = transfer_buffer.map(device, false);
    mem.mem_mut().copy_from_slice(data);
    mem.unmap();
    let location = TransferBufferLocation::default()
        .with_offset(0)
        .with_transfer_buffer(&transfer_buffer);
    let region = BufferRegion::default()
        .with_buffer(vertex_buffer)
        .with_offset(0)
        .with_size(size);
    copy_pass.upload_to_gpu_buffer(location, region, false);
    Ok(())
}