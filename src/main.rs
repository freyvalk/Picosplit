#![deny(missing_debug_implementations, missing_docs, clippy::all)]
//! Minimalist speedrun timer

mod cfg;
mod gfx;

use {
    cfg::Cfg,
    clap::{load_yaml, App},
    gfx::{init_vk_window, print_vk_ques},
    std::path::Path,
    vulkano::{
        buffer::{BufferUsage, CpuAccessibleBuffer},
        command_buffer::{AutoCommandBufferBuilder, CommandBuffer},
        device::{Device, DeviceExtensions, Features},
        instance::{Instance, InstanceExtensions, PhysicalDevice},
        sync::GpuFuture,
    },
    vulkano_win::VkSurfaceBuild,
    winit::{
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
    },
};

fn main() {
    // Parse command line args
    let cli_argfile = load_yaml!("../res/sys/cli.yaml");
    let _matches = App::from(cli_argfile).get_matches();

    let _cfg = Cfg::init_cfg(Path::new("./res/cfg/cfg.toml"));

    // Create Vulkano Instance
    // TODO Nicer no vulkan output
    let instance = {
        let extensions = vulkano_win::required_extensions();
        Instance::new(None, &extensions, None).expect("failed to create Vulkan instance")
    };

    // Setup vulkan
    let (events_loop, surface) = init_vk_window(&EventLoop::new(), instance.clone());

    // Get vk physical
    let physical = PhysicalDevice::enumerate(&instance)
        .next()
        .expect("No device available");

    print_vk_ques(&physical);

    // Get que family that supports Graphics
    let queue_family = physical
        .queue_families()
        .find(|&q| q.supports_graphics())
        .expect("couldn't find a graphical queue family");

    // Get device and queues to render to
    let (device, mut queues) = {
        Device::new(
            physical,
            &Features::none(),
            &DeviceExtensions::none(),
            [(queue_family, 0.5)].iter().cloned(),
        )
        .expect("failed to create device")
    };

    // Get Queue to render to
    let queue = queues.next().unwrap();

    // Create content to buffer.
    let data_iter = 0..65536;
    let data_buffer =
        CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), false, data_iter)
            .expect("failed to create buffer");

    // Create command buffer
    let mut builder = AutoCommandBufferBuilder::new(device.clone(), queue.family()).unwrap();

    // Copy data into buffer
    builder
        .copy_buffer(data_buffer.clone(), data_buffer)
        .unwrap();

    // Build buffer
    let command_buffer = builder.build().unwrap();

    // Execute command buffer
    let finished = command_buffer.execute(queue.clone()).unwrap();

    // Check compute operation completed successfully
    finished
        .then_signal_fence_and_flush()
        .unwrap()
        .wait(None)
        .unwrap();

    // Main program loop
    #[allow(clippy::single_match)]
    events_loop.run(|event, _, control_flow| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            *control_flow = ControlFlow::Exit;
        }
        _ => (),
    });
}
