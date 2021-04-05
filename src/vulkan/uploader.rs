use std::{mem::size_of, rc::Rc};

use ash::{version::DeviceV1_0, vk};

const DEFAULT_STAGING_BUFFER_SIZE: u64 = 16 * 1024 * 1024;

struct StagingBuffer {
    buffer: vk::Buffer,
    alloc: vk_mem::Allocation,
    mapping: *mut u8,
    pos: u64,
    size: u64,
    last_used_frame: u64,
}

pub struct Uploader {
    device: Rc<ash::Device>,
    allocator: Rc<vk_mem::Allocator>,
    staging_buffers: Vec<StagingBuffer>,
    frame_counter: u64,
    max_frames_ahead: u64,
    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,
    fences: Vec<vk::Fence>,
}

impl Uploader {
    pub fn new(device: Rc<ash::Device>, allocator: Rc<vk_mem::Allocator>, max_frames_ahead: u64, queue_family: u32) -> Uploader {
        let pool_info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(queue_family)
            .build();
        
        let command_pool = unsafe{ device.create_command_pool(&pool_info, None) }.unwrap();

        let alloc_info = vk::CommandBufferAllocateInfo::builder()
            .command_buffer_count(max_frames_ahead as u32)
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .build();
        let command_buffers = unsafe{device.allocate_command_buffers(&alloc_info)}.unwrap();

        let fence_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED).build();
        let mut fences = Vec::with_capacity(max_frames_ahead as usize);
        for _ in 0..max_frames_ahead as usize {
            fences.push(unsafe{device.create_fence(&fence_info, None)}.unwrap());
        }
        
        let res = Uploader{
            device,
            allocator,
            staging_buffers: Vec::new(),
            frame_counter: 0,
            max_frames_ahead,
            command_pool,
            command_buffers,
            fences,
        };
        
        let command_buffer = res.command_buffers[0];

        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
            .build();
        unsafe {
            res.device.begin_command_buffer(command_buffer, &begin_info).unwrap();
            res.device.reset_fences(&[res.fences[0]]).unwrap();
        }

        res
    }

    pub fn destroy(&mut self) {
        for fence in &self.fences {
            unsafe{self.device.destroy_fence(*fence, None);}
        }

        for buf in &self.staging_buffers {
            self.allocator.unmap_memory(&buf.alloc);
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
        let buffer_info = vk::BufferCreateInfo::builder()
            .size(new_size)
            .usage(vk::BufferUsageFlags::TRANSFER_SRC)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .build();
        let alloc_info = vk_mem::AllocationCreateInfo {
            usage: vk_mem::MemoryUsage::CpuOnly,
            ..Default::default()
        };

        let (buffer, alloc, _) = self.allocator.create_buffer(&buffer_info, &alloc_info).unwrap();
        let mapping = self.allocator.map_memory(&alloc).unwrap();

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

    pub fn enqueue_buffer_upload<T>(&mut self, dest_buffer: vk::Buffer, dst_offset: u64, data: &[T]) {
        let size = size_of::<T>() as u64 * data.len() as u64;
        let staging_buffer_index = self.find_staging_buffer(size);
        let staging_buffer = &mut self.staging_buffers[staging_buffer_index];

        let command_buffer = self.command_buffers[(self.frame_counter % self.max_frames_ahead) as usize];

        unsafe {
            staging_buffer.mapping.offset(staging_buffer.pos as isize).copy_from_nonoverlapping(data.as_ptr() as *const u8, size as usize);

            let regions = [
                vk::BufferCopy {
                    src_offset: staging_buffer.pos,
                    dst_offset,
                    size,
                }
            ];
            self.device.cmd_copy_buffer(command_buffer, staging_buffer.buffer, dest_buffer, &regions);
        }

        staging_buffer.pos += size;
        staging_buffer.last_used_frame = self.frame_counter;
    }

    pub fn submit_uploads(&mut self, queue: vk::Queue) {
        let command_buffer = self.command_buffers[(self.frame_counter % self.max_frames_ahead) as usize];

        unsafe {
            let mem_barrier = vk::MemoryBarrier::builder()
                .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                .dst_access_mask(vk::AccessFlags::empty())
                .build();
            self.device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::DependencyFlags::BY_REGION,
                &[mem_barrier],
                &[],
                &[]
            );
            self.device.end_command_buffer(command_buffer).unwrap();
        }

        let command_buffers = [command_buffer];
        let submit_info = vk::SubmitInfo::builder()
            .command_buffers(&command_buffers)
            .build();
        unsafe {
            self.device.queue_submit(queue, &[submit_info], self.fences[(self.frame_counter % self.max_frames_ahead) as usize]).unwrap();
        }

        self.frame_counter += 1;

        let command_buffer = self.command_buffers[(self.frame_counter % self.max_frames_ahead) as usize];
        let fence = self.fences[(self.frame_counter % self.max_frames_ahead) as usize];

        unsafe {
            self.device.wait_for_fences(&[fence], true, u64::MAX).unwrap();
            self.device.reset_fences(&[fence]).unwrap();
        }

        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
            .build();
        unsafe {
            self.device.begin_command_buffer(command_buffer, &begin_info).unwrap();
        }
    }
}
