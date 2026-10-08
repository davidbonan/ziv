use ziv::develop::domain::development::Development;
use ziv::engine::infrastructure::display_stage::DisplayRequest;
use ziv::engine::infrastructure::engine::Engine;

pub fn headless_engine() -> Engine {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .expect("a GPU adapter is available");
    let (device, queue) =
        pollster::block_on(adapter.request_device(&Engine::device_descriptor(&adapter)))
            .expect("the adapter provides a device");
    Engine::new(device, queue)
}

/// The whole source developed with `development`, at `size`.
pub fn whole_source_request(development: &Development, size: [u32; 2]) -> DisplayRequest {
    DisplayRequest {
        development: development.clone(),
        ..DisplayRequest::whole_source(size)
    }
}
