use ash::{version::DeviceV1_0, vk};
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;

use vulkan_renderer::renderer;

struct FpsTracker {
    median: f32,
    frames: u64,
    instance: std::time::Instant,
}

impl FpsTracker {
    fn new() -> Self {
        Self {
            median: 0.0,
            frames: 0,
            instance: std::time::Instant::now(),
        }
    }

    fn update(&mut self) -> bool {
        self.frames += 1;

        let elapsed = self.instance.elapsed();
        if elapsed.as_secs() > 1 {
            self.median += self.frames as f32 / elapsed.as_secs_f32();
            self.median /= 2.;
            self.frames = 0;
            self.instance = std::time::Instant::now();
            true
        } else {
            false
        }
    }

    fn fps(&self) -> f32 {
        self.median
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // setting up logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace")).init();

    let mut fps_tracker = FpsTracker::new();

    // https://hoj-senna.github.io/ashen-renderer/text/008_Cleanup.html
    let eventloop = EventLoop::new();

    let window = renderer::DEFAULT_WINDOW_INFO
        .clone()
        .into_window(&eventloop)
        .unwrap();
    let mut renderer = renderer::Renderer::init(window)?;

    eventloop.run(move |event, _, controlflow| {
        *controlflow = winit::event_loop::ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *controlflow = winit::event_loop::ControlFlow::Exit;
            }
            Event::MainEventsCleared => {
                // doing the work here (later)
                renderer.window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                let (image_index, _) = unsafe {
                    renderer
                        .swapchain
                        .swapchain_loader
                        .acquire_next_image(
                            renderer.swapchain.swapchain,
                            std::u64::MAX,
                            renderer.swapchain.image_available[renderer.swapchain.current_image],
                            vk::Fence::null(),
                        )
                        .expect("image acquisition trouble")
                };
                unsafe {
                    renderer
                        .device
                        .wait_for_fences(
                            &[renderer.swapchain.may_begin_drawing
                                [renderer.swapchain.current_image]],
                            true,
                            std::u64::MAX,
                        )
                        .expect("fence-waiting");
                    renderer
                        .device
                        .reset_fences(&[
                            renderer.swapchain.may_begin_drawing[renderer.swapchain.current_image]
                        ])
                        .expect("resetting fences");
                }
                let semaphores_available =
                    [renderer.swapchain.image_available[renderer.swapchain.current_image]];
                let waiting_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
                let semaphores_finished =
                    [renderer.swapchain.rendering_finished[renderer.swapchain.current_image]];
                let commandbuffers = [renderer.commandbuffers[image_index as usize]];
                let submit_info = [vk::SubmitInfo::builder()
                    .wait_semaphores(&semaphores_available)
                    .wait_dst_stage_mask(&waiting_stages)
                    .command_buffers(&commandbuffers)
                    .signal_semaphores(&semaphores_finished)
                    .build()];
                unsafe {
                    renderer
                        .device
                        .queue_submit(
                            renderer.queues.graphics_queue,
                            &submit_info,
                            renderer.swapchain.may_begin_drawing[renderer.swapchain.current_image],
                        )
                        .expect("queue submission");
                };
                let swapchains = [renderer.swapchain.swapchain];
                let indices = [image_index];
                let present_info = vk::PresentInfoKHR::builder()
                    .wait_semaphores(&semaphores_finished)
                    .swapchains(&swapchains)
                    .image_indices(&indices);
                unsafe {
                    renderer
                        .swapchain
                        .swapchain_loader
                        .queue_present(renderer.queues.graphics_queue, &present_info)
                        .expect("queue presentation");
                };
                renderer.swapchain.current_image = (renderer.swapchain.current_image + 1)
                    % renderer.swapchain.amount_of_images as usize;

                if fps_tracker.update() {
                    println!("FPS: {}", fps_tracker.fps());
                }
            }
            _ => {}
        }
    });
}
