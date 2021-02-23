use ash::{version::DeviceV1_0, vk};
use crystal::prelude::*;
use winit::event::{DeviceEvent, Event, WindowEvent};
use winit::event_loop::EventLoop;

use vulkan_engine::{
    color::Color,
    renderer::{
        self,
        camera::Camera,
        light::{DirectionalLight, LightManager, PointLight},
        model::{DefaultModel, InstanceData},
    },
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
    let window_size = window.inner_size();
    let mut renderer = renderer::Renderer::init(window)?;

    let mut lights = LightManager::default();
    lights.add_light(DirectionalLight {
        direction: Vec3::new(0., -1., 0.),
        illuminance: Vec3::new(10.1, 10.1, 10.1),
    });
    lights.add_light(DirectionalLight {
        direction: Vec3::new(0., 1., 0.),
        illuminance: Vec3::new(1.6, 1.6, 1.6),
    });
    lights.add_light(PointLight {
        position: Vec3::new(0.1, -3.0, -3.0),
        luminous_flux: Vec3::new(100.0, 100.0, 100.0),
    });
    lights.add_light(PointLight {
        position: Vec3::new(0.1, -3.0, -3.0),
        luminous_flux: Vec3::new(100.0, 100.0, 100.0),
    });
    lights.add_light(PointLight {
        position: Vec3::new(0.1, -3.0, -3.0),
        luminous_flux: Vec3::new(100.0, 100.0, 100.0),
    });
    lights.add_light(PointLight {
        position: Vec3::new(0.1, -3.0, -3.0),
        luminous_flux: Vec3::new(100.0, 100.0, 100.0),
    });

    lights.update_buffer(
        &renderer.device,
        &renderer.allocator,
        &mut renderer.light_buffer,
        &mut renderer.descriptor_sets_light,
    )?;

    let mut model = DefaultModel::sphere(4);

    let mut angle = 7.0.deg();

    let model_ref = model.insert_visibly(InstanceData::from_matrix_color_metallic_roughness(
        &Mat4::rotation_x(angle) * &Mat4::scale(0.1),
        Color::rgb_f32(0.955, 0.638, 0.538),
        1.0,
        0.2,
    ));

    for i in 0..10 {
        for j in 0..10 {
            model.insert_visibly(InstanceData::from_matrix_color_metallic_roughness(
                &Mat4::translate(Vec3::new(i as f32 - 5.0, -j as f32 + 5.0, 10.0))
                    * &Mat4::scale(0.5),
                Color::rgb_f32(1.0, 0.86, 0.57),
                i as f32 * 0.1,
                j as f32 * 0.1,
            ));
        }
    }

    for i in 0..10 {
        model.insert_visibly(InstanceData::from_matrix_color_metallic_roughness(
            &Mat4::translate(Vec3::new(i as f32 - 5.0, -6.0, 10.0)) * &Mat4::scale(0.5),
            Color::rgb_f32(
                1.0 * i as f32 * 0.1,
                0.0 * i as f32 * 0.1,
                0.0 * i as f32 * 0.1,
            ),
            0.5,
            0.5,
        ));
    }

    model.update_vertex_buffer(&renderer.allocator).unwrap();
    model.update_index_buffer(&renderer.allocator).unwrap();
    model.update_instance_buffer(&renderer.allocator).unwrap();

    renderer.models.push(model);
    // let texture_id = renderer.new_texture_from_file("./assets/images/rust.png")?;

    // if let Some(texture) = renderer.texture_storage.get(texture_id) {
    //     for dss in &renderer.descriptor_sets_texture {
    //         let imageinfo = vk::DescriptorImageInfo {
    //             image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    //             image_view: texture.imageview,
    //             sampler: texture.sampler,
    //         };
    //         let descriptorwrite_image = vk::WriteDescriptorSet {
    //             dst_set: *dss,
    //             dst_binding: 0,
    //             dst_array_element: 0,
    //             descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
    //             descriptor_count: 1,
    //             p_image_info: [imageinfo].as_ptr(),
    //             ..Default::default()
    //         };

    //         unsafe {
    //             renderer
    //                 .device
    //                 .update_descriptor_sets(&[descriptorwrite_image], &[]);
    //         }
    //     }
    // }

    // let mut model = TextureQuadModel::quad();
    // model.insert_visibly(TexturedInstanceData::from_matrix(Mat4::identity()));

    // model.update_vertex_buffer(&renderer.allocator).unwrap();
    // model.update_index_buffer(&renderer.allocator).unwrap();
    // model.update_instance_buffer(&renderer.allocator).unwrap();

    // renderer.texture_quads.push(model);

    let mut camera = Camera::builder()
        //.fovy(30.0.deg())
        .position(Vec3::new(0.0, 0.0, -5.0))
        .aspect(window_size.width as f32 / window_size.height as f32)
        .build();

    let mut last_time = std::time::Instant::now();
    let mut fwd = false;
    let mut back = false;
    let mut right = false;
    let mut left = false;

    let mut cursor_catched = false;

    eventloop.run(move |event, _, controlflow| {
        *controlflow = winit::event_loop::ControlFlow::Poll;

        let delta = last_time.elapsed().as_nanos() as f32 / 1000000000.0f32;
        last_time = std::time::Instant::now();

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *controlflow = winit::event_loop::ControlFlow::Exit,
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta: motion },
                ..
            } => {
                if cursor_catched {
                    camera.rotate(
                        Angle::from_deg(motion.1 as f32 * 0.5),
                        Angle::from_deg(motion.0 as f32 * 0.5),
                    );
                }
            }
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            winit::event::KeyboardInput {
                                state,
                                virtual_keycode: Some(keycode),
                                ..
                            },
                        ..
                    },
                ..
            } => match keycode {
                winit::event::VirtualKeyCode::Escape => {
                    if state == winit::event::ElementState::Pressed {
                        cursor_catched = !cursor_catched;
                        if cursor_catched {
                            renderer.window.set_cursor_visible(false);
                            renderer.window.set_cursor_grab(true);
                        } else {
                            renderer.window.set_cursor_visible(true);
                            renderer.window.set_cursor_grab(false);
                        }
                    }
                }
                winit::event::VirtualKeyCode::Up | winit::event::VirtualKeyCode::W => {
                    fwd = state == winit::event::ElementState::Pressed;
                }
                winit::event::VirtualKeyCode::Down | winit::event::VirtualKeyCode::S => {
                    back = state == winit::event::ElementState::Pressed;
                }
                winit::event::VirtualKeyCode::A | winit::event::VirtualKeyCode::Left => {
                    left = state == winit::event::ElementState::Pressed;
                }
                winit::event::VirtualKeyCode::D | winit::event::VirtualKeyCode::Right => {
                    right = state == winit::event::ElementState::Pressed;
                }
                winit::event::VirtualKeyCode::R => {
                    renderer.recreate_swapchain().expect("swapchain recreation");
                }
                winit::event::VirtualKeyCode::F12 => {
                    renderer::screenshot(&renderer).expect("screenshot trouble");
                }
                winit::event::VirtualKeyCode::Q => {
                    *controlflow = winit::event_loop::ControlFlow::Exit;
                }
                _ => {}
            },
            Event::MainEventsCleared => {
                // doing the work here (later)
                angle = Angle::from_deg(angle.to_deg() + 0.01);

                let new_model_matrix = &Mat4::rotation_x(angle) * &Mat4::scale(0.1);

                renderer.models[0].get_mut(model_ref).unwrap().model_matrix = new_model_matrix;
                renderer.models[0]
                    .get_mut(model_ref)
                    .unwrap()
                    .inverse_model_matrix = new_model_matrix.try_inverse().unwrap();

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
                    match renderer
                        .swapchain
                        .swapchain_loader
                        .queue_present(renderer.queues.present_queue, &present_info)
                    {
                        Ok(..) => {}
                        Err(ash::vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                            renderer.recreate_swapchain().expect("swapchain recreation");
                            camera.set_aspect(
                                renderer.swapchain.extent.width as f32
                                    / renderer.swapchain.extent.height as f32,
                            );
                            camera.update_buffer(&renderer.allocator, &mut renderer.uniform_buffer);
                        }
                        _ => {
                            panic!("unhandled queue presentation error");
                        }
                    }
                };
                renderer.swapchain.current_image = (renderer.swapchain.current_image + 1)
                    % renderer.swapchain.amount_of_images as usize;

                if fps_tracker.update() {
                    println!("FPS: {}", fps_tracker.fps());
                }
            }
            _ => {}
        }

        let mut movement = Vec3::new(0.0f32, 0.0f32, 0.0f32);
        if fwd {
            movement += Vec3::new(0.0, 0.0, 1.0);
        }
        if back {
            movement += Vec3::new(0.0, 0.0, -1.0);
        }
        if right {
            movement += Vec3::new(1.0, 0.0, 0.0);
        }
        if left {
            movement += Vec3::new(-1.0, 0.0, 0.0);
        }
        camera.move_in_view_direction(&(&movement * (5.0 * delta)));
    });
}
