use {
    anyhow::{anyhow, Context, Result},
    std::borrow::Cow,
    wgpu::util::DeviceExt,
};

fn load_shader_src() -> &'static str {
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/shader.wgsl"))
}

async fn run() -> Result<u32> {
    let instance = wgpu::Instance::default();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await
        .context("Failed to initialize adapter")?;
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default().using_resolution(adapter.limits()),
            },
            None,
        )
        .await
        .context("Failed to initialize device")?;

    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(load_shader_src())),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: None,
        module: &shader_module,
        entry_point: "main",
    });

    let data: u32 = 0;
    let flag_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Storage Buffer"),
        contents: bytemuck::bytes_of(&data),
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let output_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Storage Buffer"),
        contents: bytemuck::bytes_of(&data),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    });
    let output_staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: std::mem::size_of::<u32>() as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let bind_group_layout = pipeline.get_bind_group_layout(0);
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: flag_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: output_buffer.as_entire_binding(),
            },
        ],
    });

    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
        cpass.set_pipeline(&pipeline);
        cpass.set_bind_group(0, &bind_group, &[]);
        cpass.dispatch_workgroups(10, 1, 1);
    }
    encoder.copy_buffer_to_buffer(
        &output_buffer,
        0,
        &output_staging_buffer,
        0,
        std::mem::size_of::<u32>() as u64,
    );
    queue.submit(Some(encoder.finish()));

    let buffer_slice = output_staging_buffer.slice(..);
    let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());
    device.poll(wgpu::Maintain::Wait);

    if let Some(Ok(())) = receiver.receive().await {
        let data = buffer_slice.get_mapped_range();
        let result: u32 = *bytemuck::from_bytes(&data);
        drop(data);
        output_staging_buffer.unmap();

        Ok(result)
    } else {
        Err(anyhow!("failed to run compute on gpu!"))
    }
}

pub fn run_native() -> Result<()> {
    let result = pollster::block_on(run())?;
    println!("Result: {}", result);
    Ok(())
}

#[cfg(target_arch = "wasm32")]
mod web {
    use {futures::future::TryFutureExt, wasm_bindgen::prelude::*};

    #[wasm_bindgen]
    pub fn run_test() -> js_sys::Promise {
        wasm_bindgen_futures::future_to_promise(
            super::run()
                .map_ok(|value| JsValue::from_f64(value as f64))
                .map_err(|e| JsValue::from_str(&e.to_string())),
        )
    }
}
