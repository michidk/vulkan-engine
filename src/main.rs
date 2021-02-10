use ash::{version::DeviceV1_0, vk};
use math::prelude::*;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;

use vulkan_engine::{
    color::Color,
    renderer::{self, Camera, DefaultModel, InstanceData},
};

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
    let mut cube = DefaultModel::cube();

    let mut angle = 7.0.deg();

    let cube_x = cube.insert_visibly(InstanceData {
        position: dbg!(
            &(&Mat4::new_translate(Vec3::new(0.05, 0.05, 0.0)) * &Mat4::new_rotation_x(angle))
                * &Mat4::new_scaling(0.1)
        ),
        color: Color::rgb_f32(1.0, 1.0, 0.2),
    });

    cube.insert_visibly(InstanceData {
        position: dbg!(
            &(&Mat4::new_translate(Vec3::new(0.20, 0.20, 0.1)) * &Mat4::new_rotation_z(10.0.deg()))
                * &Mat4::new_scaling(0.1)
        ),
        color: Color::rgb_f32(0.6, 0.2, 0.2),
    });

    cube.insert_visibly(InstanceData {
        position: dbg!(
            &(&Mat4::new_translate(Vec3::new(0.80, 1.0, -0.4)) * &Mat4::new_rotation_z(25.0.deg()))
                * &Mat4::new_scaling(0.12)
        ),
        color: Color::rgb_f32(0.6, 0.2, 0.2),
    });

    cube.insert_visibly(InstanceData {
        position: dbg!(
            &(&Mat4::new_translate(Vec3::new(0.20, 0.20, 0.1)) * &Mat4::new_rotation_z(10.0.deg()))
                * &Mat4::new_scaling(0.1)
        ),
        color: Color::rgb_f32(0.6, 0.2, 0.2),
    });

    cube.update_vertex_buffer(&renderer.allocator).unwrap();
    cube.update_index_buffer(&renderer.allocator).unwrap();
    cube.update_instance_buffer(&renderer.allocator).unwrap();

    renderer.models.push(cube);

    let mut camera = Camera::builder()
        .near(0.3)
        .far(1000.0)
        .fovy(30.0.deg())
        .aspect(1920.0 / 1080.0)
        .position(Vec3::new(0.0, 3.0, 0.0))
        .view_direction(Vec3::new(0.0, -1.0, 0.0))
        .build();

    eventloop.run(move |event, _, controlflow| {
        *controlflow = winit::event_loop::ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *controlflow = winit::event_loop::ControlFlow::Exit,
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            winit::event::KeyboardInput {
                                state: winit::event::ElementState::Pressed,
                                virtual_keycode: Some(keycode),
                                ..
                            },
                        ..
                    },
                ..
            } => match keycode {
                winit::event::VirtualKeyCode::Up | winit::event::VirtualKeyCode::W => {
                    camera.move_forward(0.05);
                }
                winit::event::VirtualKeyCode::Down | winit::event::VirtualKeyCode::S => {
                    camera.move_backward(0.05);
                }
                winit::event::VirtualKeyCode::Left | winit::event::VirtualKeyCode::A => {
                    camera.turn_left(0.1.rad());
                }
                winit::event::VirtualKeyCode::Right | winit::event::VirtualKeyCode::D => {
                    camera.turn_right(0.1.rad());
                }
                winit::event::VirtualKeyCode::PageUp => {
                    camera.turn_up(0.02.rad());
                }
                winit::event::VirtualKeyCode::PageDown => {
                    camera.turn_down(0.02.rad());
                }
                _ => {}
            },
            Event::MainEventsCleared => {
                // doing the work here (later)
                angle = Angle::from_deg(angle.to_deg() + 0.01);
                renderer.models[0].get_mut(cube_x).unwrap().position =
                    &(&Mat4::new_translate(Vec3::new(0.05, 0.05, 0.0))
                        * &Mat4::new_rotation_z(angle))
                        * &Mat4::new_scaling(0.1);
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
                camera.update_buffer(&renderer.allocator, &mut renderer.uniform_buffer);
                for m in &mut renderer.models {
                    m.update_instance_buffer(&renderer.allocator).unwrap();
                }
                renderer
                    .update_commandbuffer(image_index as usize)
                    .expect("updating the command buffer");

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
