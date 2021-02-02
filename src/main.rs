use ash::{version::DeviceV1_0, vk};
use winit::event_loop::EventLoop;

use vulkan_renderer::engine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // setting up logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace")).init();

    // https://hoj-senna.github.io/ashen-engine/text/008_Cleanup.html
    let eventloop = EventLoop::new();
    let window = engine::DEFAULT_WINDOW_INFO.clone().into_window(&eventloop).unwrap();
    let mut engine = engine::Engine::init(window)?;
    use winit::event::{Event, WindowEvent};
    eventloop.run(move |event, _, controlflow| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            *controlflow = winit::event_loop::ControlFlow::Exit;
        }
        Event::MainEventsCleared => {
            // doing the work here (later)
            engine.window.request_redraw();
        }
        Event::RedrawRequested(_) => {
            let (image_index, _) = unsafe {
                engine
                    .swapchain
                    .swapchain_loader
                    .acquire_next_image(
                        engine.swapchain.swapchain,
                        std::u64::MAX,
                        engine.swapchain.image_available[engine.swapchain.current_image],
                        vk::Fence::null(),
                    )
                    .expect("image acquisition trouble")
            };
            unsafe {
                engine
                    .device
                    .wait_for_fences(
                        &[engine.swapchain.may_begin_drawing[engine.swapchain.current_image]],
                        true,
                        std::u64::MAX,
                    )
                    .expect("fence-waiting");
                engine
                    .device
                    .reset_fences(&[
                        engine.swapchain.may_begin_drawing[engine.swapchain.current_image]
                    ])
                    .expect("resetting fences");
            }
            let semaphores_available =
                [engine.swapchain.image_available[engine.swapchain.current_image]];
            let waiting_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            let semaphores_finished =
                [engine.swapchain.rendering_finished[engine.swapchain.current_image]];
            let commandbuffers = [engine.commandbuffers[image_index as usize]];
            let submit_info = [vk::SubmitInfo::builder()
                .wait_semaphores(&semaphores_available)
                .wait_dst_stage_mask(&waiting_stages)
                .command_buffers(&commandbuffers)
                .signal_semaphores(&semaphores_finished)
                .build()];
            unsafe {
                engine
                    .device
                    .queue_submit(
                        engine.queues.graphics_queue,
                        &submit_info,
                        engine.swapchain.may_begin_drawing[engine.swapchain.current_image],
                    )
                    .expect("queue submission");
            };
            let swapchains = [engine.swapchain.swapchain];
            let indices = [image_index];
            let present_info = vk::PresentInfoKHR::builder()
                .wait_semaphores(&semaphores_finished)
                .swapchains(&swapchains)
                .image_indices(&indices);
            unsafe {
                engine
                    .swapchain
                    .swapchain_loader
                    .queue_present(engine.queues.graphics_queue, &present_info)
                    .expect("queue presentation");
            };
            engine.swapchain.current_image =
                (engine.swapchain.current_image + 1) % engine.swapchain.amount_of_images as usize;
        }
        _ => {}
    });
}
