use std::{mem::size_of, rc::Rc};

use ash::{extensions::khr, vk};
use crystal::prelude::Mat4;
use gpu_allocator::{MemoryLocation, SubAllocation};
use ve_format::mesh::Vertex;

use super::allocator::Allocator;

const DEFAULT_STAGING_BUFFER_SIZE: u64 = 16 * 1024 * 1024;

struct StagingBuffer {
    buffer: vk::Buffer,
    alloc: SubAllocation,
    mapping: *mut u8,
    pos: u64,
    size: u64,
    last_used_frame: u64,
}

#[derive(Clone)]
struct ScratchBuffer {
    buffer: vk::Buffer,
    alloc: SubAllocation,
}

/// This struct automatically manages a pool of staging buffers that can be used to upload data to GPU-only buffers and images.
///
/// # Notes
/// This struct has to be cleaned up manually by calling [`destroy()`](Uploader::destroy()).
pub struct Uploader {
    device: Rc<ash::Device>,
    allocator: Rc<Allocator>,
    staging_buffers: Vec<StagingBuffer>,
    frame_counter: u64,
    max_frames_ahead: u64,
    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,
    fences: Vec<vk::Fence>,
    scratch_buffers: Vec<Vec<ScratchBuffer>>,
}

impl Uploader {
    /// Creates a new [`Uploader`].
    pub fn new(
        device: Rc<ash::Device>,
        allocator: Rc<Allocator>,
        max_frames_ahead: u64,
        queue_family: u32,
    ) -> Uploader {
        let pool_info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(queue_family)
            .build();

        let command_pool = unsafe { device.create_command_pool(&pool_info, None) }.unwrap();

        let alloc_info = vk::CommandBufferAllocateInfo::builder()
            .command_buffer_count(max_frames_ahead as u32)
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .build();
        let command_buffers = unsafe { device.allocate_command_buffers(&alloc_info) }.unwrap();

        let fence_info = vk::FenceCreateInfo::builder()
            .flags(vk::FenceCreateFlags::SIGNALED)
            .build();
        let mut fences = Vec::with_capacity(max_frames_ahead as usize);
        for _ in 0..max_frames_ahead as usize {
            fences.push(unsafe { device.create_fence(&fence_info, None) }.unwrap());
        }

        let res = Uploader {
            device,
            allocator,
            staging_buffers: Vec::new(),
            frame_counter: 0,
            max_frames_ahead,
            command_pool,
            command_buffers,
            fences,
            scratch_buffers: vec![vec![]; max_frames_ahead as usize],
        };

        let command_buffer = res.command_buffers[0];

        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
            .build();
        unsafe {
            res.device
                .begin_command_buffer(command_buffer, &begin_info)
                .unwrap();
            res.device.reset_fences(&[res.fences[0]]).unwrap();
        }

        res
    }

    /// Destroys a [`Uploader`].
    ///
    /// The object *must not* be used after calling this method.
    pub fn destroy(&mut self) {
        for fence in &self.fences {
            unsafe {
                self.device.destroy_fence(*fence, None);
            }
        }

        for buf in &self.staging_buffers {
            self.allocator.destroy_buffer(buf.buffer, &buf.alloc);
        }

        unsafe {
            self.device.destroy_command_pool(self.command_pool, None);
        }
    }

    fn find_staging_buffer(&mut self, size: u64) -> usize {
        for (i, sb) in self.staging_buffers.iter_mut().enumerate() {
            // staging buffer was not used in any frame that might still be in flight, reset pos
            if self.frame_counter - sb.last_used_frame >= self.max_frames_ahead {
                sb.pos = 0;
            }

            if sb.size - sb.pos >= size {
                return i;
            }
        }

        // no staging buffer with enough capacity found, create a new one
        let new_size = size.max(DEFAULT_STAGING_BUFFER_SIZE);
        let (buffer, alloc) = self.allocator.create_buffer(
            new_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            MemoryLocation::CpuToGpu,
        );
        let mapping = Allocator::get_ptr(&alloc) as *mut u8;

        self.staging_buffers.push(StagingBuffer {
            buffer,
            alloc,
            mapping,
            pos: 0,
            size: new_size,
            last_used_frame: 0,
        });

        self.staging_buffers.len() - 1
    }

    pub fn enqueue_scene_acc_struct_build(
        &mut self,
        rtx_ext: Rc<khr::AccelerationStructure>,
        objects: &[(vk::AccelerationStructureKHR, Mat4<f32>)],
    ) -> (vk::AccelerationStructureKHR, vk::Buffer, SubAllocation) {
        log::info!(
            "Building Scene AccelerationStructure for {} objects",
            objects.len()
        );

        let (staging_buffer, staging_buffer_alloc) = self.allocator.create_buffer(
            (size_of::<vk::AccelerationStructureInstanceKHR>() * objects.len()) as u64,
            vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR
                | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
            MemoryLocation::CpuToGpu,
        );
        let ptr =
            Allocator::get_ptr(&staging_buffer_alloc) as *mut vk::AccelerationStructureInstanceKHR;
        for (i, (acc, transform)) in objects.iter().enumerate() {
            let acc_addr = unsafe {
                rtx_ext.get_acceleration_structure_device_address(
                    &vk::AccelerationStructureDeviceAddressInfoKHR::builder()
                        .acceleration_structure(*acc)
                        .build(),
                )
            };

            unsafe {
                ptr.add(i).write(vk::AccelerationStructureInstanceKHR {
                    transform: vk::TransformMatrixKHR {
                        matrix: [
                            *transform.get_unchecked((0, 0)),
                            *transform.get_unchecked((0, 1)),
                            *transform.get_unchecked((0, 2)),
                            *transform.get_unchecked((0, 3)),
                            *transform.get_unchecked((1, 0)),
                            *transform.get_unchecked((1, 1)),
                            *transform.get_unchecked((1, 2)),
                            *transform.get_unchecked((1, 3)),
                            *transform.get_unchecked((2, 0)),
                            *transform.get_unchecked((2, 1)),
                            *transform.get_unchecked((2, 2)),
                            *transform.get_unchecked((2, 3)),
                        ],
                    },
                    instance_custom_index_and_mask: 0xFF000000,
                    instance_shader_binding_table_record_offset_and_flags: 0,
                    acceleration_structure_reference: vk::AccelerationStructureReferenceKHR {
                        device_handle: acc_addr,
                    },
                })
            };
        }
        let staging_buffer_addr = unsafe {
            self.device.get_buffer_device_address(
                &vk::BufferDeviceAddressInfo::builder()
                    .buffer(staging_buffer)
                    .build(),
            )
        };

        let geometries = [vk::AccelerationStructureGeometryKHR::builder()
            .geometry_type(vk::GeometryTypeKHR::INSTANCES)
            .geometry(vk::AccelerationStructureGeometryDataKHR {
                instances: vk::AccelerationStructureGeometryInstancesDataKHR::builder()
                    .array_of_pointers(false)
                    .data(vk::DeviceOrHostAddressConstKHR {
                        device_address: staging_buffer_addr,
                    })
                    .build(),
            })
            .build()];
        let mut geometry_info = vk::AccelerationStructureBuildGeometryInfoKHR::builder()
            .ty(vk::AccelerationStructureTypeKHR::TOP_LEVEL)
            .mode(vk::BuildAccelerationStructureModeKHR::BUILD)
            .geometries(&geometries)
            .build();

        let build_size = unsafe {
            rtx_ext.get_acceleration_structure_build_sizes(
                vk::AccelerationStructureBuildTypeKHR::DEVICE,
                &geometry_info,
                &[objects.len() as u32],
            )
        };
        log::info!(
            "AccelerationStructure needs {} bytes and {} bytes of scratch space",
            build_size.acceleration_structure_size,
            build_size.build_scratch_size
        );

        let (acc_buffer, acc_buffer_alloc) = self.allocator.create_buffer(
            build_size.acceleration_structure_size,
            vk::BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR,
            MemoryLocation::GpuOnly,
        );
        let (scratch_buffer, scratch_buffer_alloc) = self.allocator.create_buffer(
            build_size.build_scratch_size,
            vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
            MemoryLocation::GpuOnly,
        );

        let acc = unsafe {
            rtx_ext
                .create_acceleration_structure(
                    &vk::AccelerationStructureCreateInfoKHR::builder()
                        .buffer(acc_buffer)
                        .offset(0)
                        .size(build_size.acceleration_structure_size)
                        .ty(vk::AccelerationStructureTypeKHR::TOP_LEVEL)
                        .build(),
                    None,
                )
                .unwrap()
        };

        let cmd = self.command_buffers[(self.frame_counter % self.max_frames_ahead) as usize];

        unsafe {
            geometry_info.scratch_data = vk::DeviceOrHostAddressKHR {
                device_address: self.device.get_buffer_device_address(
                    &vk::BufferDeviceAddressInfo::builder()
                        .buffer(scratch_buffer)
                        .build(),
                ),
            };
            geometry_info.dst_acceleration_structure = acc;

            rtx_ext.cmd_build_acceleration_structures(
                cmd,
                &[geometry_info],
                &[&[vk::AccelerationStructureBuildRangeInfoKHR::builder()
                    .primitive_count(objects.len() as u32)
                    .build()]],
            );
        }

        self.scratch_buffers[(self.frame_counter % self.max_frames_ahead) as usize].push(
            ScratchBuffer {
                buffer: scratch_buffer,
                alloc: scratch_buffer_alloc,
            },
        );
        self.scratch_buffers[(self.frame_counter % self.max_frames_ahead) as usize].push(
            ScratchBuffer {
                buffer: staging_buffer,
                alloc: staging_buffer_alloc,
            },
        );

        (acc, acc_buffer, acc_buffer_alloc)
    }

    pub fn enqueue_acc_struct_build(
        &mut self,
        rtx_ext: Rc<khr::AccelerationStructure>,
        vertex_buffer: vk::Buffer,
        index_buffer: vk::Buffer,
        submeshes: &[(u32, u32)],
        vertex_count: u32,
    ) -> (vk::AccelerationStructureKHR, vk::Buffer, SubAllocation) {
        log::info!(
            "Building AccelerationStructure for mesh with {} submeshes",
            submeshes.len()
        );

        let vertex_buffer_addr = unsafe {
            self.device.get_buffer_device_address(
                &vk::BufferDeviceAddressInfo::builder().buffer(vertex_buffer),
            )
        };
        let index_buffer_addr = unsafe {
            self.device.get_buffer_device_address(
                &vk::BufferDeviceAddressInfo::builder().buffer(index_buffer),
            )
        };

        let geometries = [vk::AccelerationStructureGeometryKHR::builder()
            .geometry_type(vk::GeometryTypeKHR::TRIANGLES)
            .geometry(vk::AccelerationStructureGeometryDataKHR {
                triangles: vk::AccelerationStructureGeometryTrianglesDataKHR::builder()
                    .vertex_format(vk::Format::R32G32B32_SFLOAT)
                    .vertex_data(vk::DeviceOrHostAddressConstKHR {
                        device_address: vertex_buffer_addr,
                    })
                    .vertex_stride(size_of::<Vertex>() as u64)
                    .max_vertex(vertex_count - 1)
                    .index_type(vk::IndexType::UINT32)
                    .index_data(vk::DeviceOrHostAddressConstKHR {
                        device_address: index_buffer_addr,
                    })
                    .build(),
            })
            .build()];
        let mut geometry_info = vk::AccelerationStructureBuildGeometryInfoKHR::builder()
            .ty(vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL)
            .mode(vk::BuildAccelerationStructureModeKHR::BUILD)
            .geometries(&geometries)
            .build();

        let build_size = unsafe {
            rtx_ext.get_acceleration_structure_build_sizes(
                vk::AccelerationStructureBuildTypeKHR::DEVICE,
                &geometry_info,
                &[submeshes[0].1],
            )
        };
        log::info!(
            "AccelerationStructure needs {} bytes and {} bytes of scratch space",
            build_size.acceleration_structure_size,
            build_size.build_scratch_size
        );

        let (acc_buffer, acc_buffer_alloc) = self.allocator.create_buffer(
            build_size.acceleration_structure_size,
            vk::BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR,
            MemoryLocation::GpuOnly,
        );
        let (scratch_buffer, scratch_buffer_alloc) = self.allocator.create_buffer(
            build_size.build_scratch_size,
            vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
            MemoryLocation::GpuOnly,
        );

        let acc = unsafe {
            rtx_ext
                .create_acceleration_structure(
                    &vk::AccelerationStructureCreateInfoKHR::builder()
                        .buffer(acc_buffer)
                        .offset(0)
                        .size(build_size.acceleration_structure_size)
                        .ty(vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL)
                        .build(),
                    None,
                )
                .unwrap()
        };

        let cmd = self.command_buffers[(self.frame_counter % self.max_frames_ahead) as usize];

        unsafe {
            // wait for vertex/index buffer transfers
            self.device.cmd_pipeline_barrier(
                cmd,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::ACCELERATION_STRUCTURE_BUILD_KHR,
                vk::DependencyFlags::BY_REGION,
                &[vk::MemoryBarrier::builder()
                    .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                    .dst_access_mask(vk::AccessFlags::TRANSFER_READ)
                    .build()],
                &[],
                &[],
            );

            geometry_info.scratch_data = vk::DeviceOrHostAddressKHR {
                device_address: self.device.get_buffer_device_address(
                    &vk::BufferDeviceAddressInfo::builder()
                        .buffer(scratch_buffer)
                        .build(),
                ),
            };
            geometry_info.dst_acceleration_structure = acc;

            rtx_ext.cmd_build_acceleration_structures(
                cmd,
                &[geometry_info],
                &[&[vk::AccelerationStructureBuildRangeInfoKHR::builder()
                    .primitive_count(submeshes[0].1 / 3)
                    .build()]],
            );
        }

        self.scratch_buffers[(self.frame_counter % self.max_frames_ahead) as usize].push(
            ScratchBuffer {
                buffer: scratch_buffer,
                alloc: scratch_buffer_alloc,
            },
        );

        (acc, acc_buffer, acc_buffer_alloc)
    }

    /// Enqueues a buffer upload command.
    ///
    /// The data upload will happend before any other vulkan commands are executed this frame.
    pub fn enqueue_buffer_upload<T>(
        &mut self,
        dest_buffer: vk::Buffer,
        dst_offset: u64,
        data: &[T],
    ) {
        let size = size_of::<T>() as u64 * data.len() as u64;
        let staging_buffer_index = self.find_staging_buffer(size);
        let staging_buffer = &mut self.staging_buffers[staging_buffer_index];

        let command_buffer =
            self.command_buffers[(self.frame_counter % self.max_frames_ahead) as usize];

        unsafe {
            staging_buffer
                .mapping
                .offset(staging_buffer.pos as isize)
                .copy_from_nonoverlapping(data.as_ptr() as *const u8, size as usize);

            let regions = [vk::BufferCopy {
                src_offset: staging_buffer.pos,
                dst_offset,
                size,
            }];
            self.device.cmd_copy_buffer(
                command_buffer,
                staging_buffer.buffer,
                dest_buffer,
                &regions,
            );
        }

        staging_buffer.pos += size;
        staging_buffer.last_used_frame = self.frame_counter;
    }

    /// Enqueues an image upload command.
    ///
    /// The image upload will happend before any other vulkan commands are executed this frame.
    ///
    /// Any previous contents of the image will be discarded. After upload,
    /// the image will be transitioned to the given `layout`.
    pub fn enqueue_image_upload(
        &mut self,
        dst_image: vk::Image,
        layout: vk::ImageLayout,
        width: u32,
        height: u32,
        pixels: &[u8],
    ) {
        let size = width as u64 * height as u64 * 4;
        let staging_buffer_index = self.find_staging_buffer(size);
        let staging_buffer = &mut self.staging_buffers[staging_buffer_index];

        let command_buffer =
            self.command_buffers[(self.frame_counter % self.max_frames_ahead) as usize];

        unsafe {
            staging_buffer
                .mapping
                .offset(staging_buffer.pos as isize)
                .copy_from_nonoverlapping(pixels.as_ptr() as *const u8, size as usize);

            let transition = vk::ImageMemoryBarrier::builder()
                .src_access_mask(vk::AccessFlags::empty())
                .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                .old_layout(vk::ImageLayout::UNDEFINED)
                .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .src_queue_family_index(0)
                .dst_queue_family_index(0)
                .image(dst_image)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                })
                .build();
            self.device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::BY_REGION,
                &[],
                &[],
                &[transition],
            );

            let regions = [vk::BufferImageCopy::builder()
                .buffer_offset(staging_buffer.pos)
                .buffer_row_length(0)
                .buffer_image_height(0)
                .image_subresource(vk::ImageSubresourceLayers {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    mip_level: 0,
                    base_array_layer: 0,
                    layer_count: 1,
                })
                .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
                .image_extent(vk::Extent3D {
                    width,
                    height,
                    depth: 1,
                })
                .build()];
            self.device.cmd_copy_buffer_to_image(
                command_buffer,
                staging_buffer.buffer,
                dst_image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &regions,
            );

            let transition = vk::ImageMemoryBarrier::builder()
                .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                .dst_access_mask(vk::AccessFlags::empty())
                .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .new_layout(layout)
                .src_queue_family_index(0)
                .dst_queue_family_index(0)
                .image(dst_image)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                })
                .build();
            self.device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::DependencyFlags::BY_REGION,
                &[],
                &[],
                &[transition],
            );
        }

        staging_buffer.pos += size;
        staging_buffer.last_used_frame = self.frame_counter;
    }

    /// Submits all upload commands for the current frame.
    ///
    /// This method should be called once per frame before any rendering takes place.
    pub fn submit_uploads(&mut self, queue: vk::Queue) {
        let command_buffer =
            self.command_buffers[(self.frame_counter % self.max_frames_ahead) as usize];

        unsafe {
            let mem_barrier = vk::MemoryBarrier::builder()
                .src_access_mask(
                    vk::AccessFlags::TRANSFER_WRITE
                        | vk::AccessFlags::ACCELERATION_STRUCTURE_WRITE_KHR,
                )
                .dst_access_mask(vk::AccessFlags::empty())
                .build();
            self.device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TRANSFER
                    | vk::PipelineStageFlags::ACCELERATION_STRUCTURE_BUILD_KHR,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::DependencyFlags::BY_REGION,
                &[mem_barrier],
                &[],
                &[],
            );
            self.device.end_command_buffer(command_buffer).unwrap();
        }

        let command_buffers = [command_buffer];
        let submit_info = vk::SubmitInfo::builder()
            .command_buffers(&command_buffers)
            .build();
        unsafe {
            self.device
                .queue_submit(
                    queue,
                    &[submit_info],
                    self.fences[(self.frame_counter % self.max_frames_ahead) as usize],
                )
                .unwrap();
        }

        self.frame_counter += 1;

        let command_buffer =
            self.command_buffers[(self.frame_counter % self.max_frames_ahead) as usize];
        let fence = self.fences[(self.frame_counter % self.max_frames_ahead) as usize];

        unsafe {
            self.device
                .wait_for_fences(&[fence], true, u64::MAX)
                .unwrap();
            self.device.reset_fences(&[fence]).unwrap();
        }

        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
            .build();
        unsafe {
            self.device
                .begin_command_buffer(command_buffer, &begin_info)
                .unwrap();
        }

        for sb in &self.scratch_buffers[(self.frame_counter % self.max_frames_ahead) as usize] {
            log::info!("Destroying scratch buffer");
            self.allocator.destroy_buffer(sb.buffer, &sb.alloc);
        }
        self.scratch_buffers[(self.frame_counter % self.max_frames_ahead) as usize].clear();
    }
}
