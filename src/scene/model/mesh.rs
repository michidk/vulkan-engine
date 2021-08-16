use std::{mem::size_of, rc::Rc};

use ash::{extensions::khr, vk};
use gpu_allocator::SubAllocation;
use ve_format::mesh::{Face, MeshData, Vertex};

use crate::vulkan::{allocator::Allocator, uploader::Uploader};

pub struct MeshRtxData {
    pub acc_struct: vk::AccelerationStructureKHR,
    pub buffer: vk::Buffer,
    pub buffer_alloc: gpu_allocator::SubAllocation,
    rtx_ext: Rc<khr::AccelerationStructure>,
}

pub struct Mesh {
    allocator: Rc<Allocator>,
    pub vertex_buffer: vk::Buffer,
    pub vertex_buffer_alloc: SubAllocation,
    pub index_buffer: vk::Buffer,
    pub index_buffer_alloc: SubAllocation,
    pub submeshes: Vec<(u32, u32)>,

    pub rtx_data: Option<MeshRtxData>,
}

impl Drop for Mesh {
    fn drop(&mut self) {
        self.allocator
            .destroy_buffer(self.vertex_buffer, &self.vertex_buffer_alloc);
        self.allocator
            .destroy_buffer(self.index_buffer, &self.index_buffer_alloc);

        if let Some(rtx_data) = &self.rtx_data {
            unsafe {
                rtx_data
                    .rtx_ext
                    .destroy_acceleration_structure(rtx_data.acc_struct, None)
            };
            self.allocator
                .destroy_buffer(rtx_data.buffer, &rtx_data.buffer_alloc);
        }
    }
}

impl Mesh {
    pub fn bake(
        mesh_data: MeshData,
        allocator: Rc<Allocator>,
        uploader: &mut Uploader,
        rtx_ext: Option<Rc<khr::AccelerationStructure>>,
    ) -> Result<Rc<Mesh>, bool> {
        let extra_flags = if rtx_ext.is_some() {
            vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR
        } else {
            vk::BufferUsageFlags::empty()
        };

        let vertex_buffer_size = mesh_data.vertices.len() * size_of::<Vertex>();
        let (vertex_buffer, vertex_buffer_alloc) = allocator.create_buffer(
            vertex_buffer_size as u64,
            vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST | extra_flags,
            gpu_allocator::MemoryLocation::GpuOnly,
        );

        let mut index_buffer_size = 0u64;
        for sm in &mesh_data.submeshes {
            index_buffer_size += (sm.faces.len() * size_of::<Face>()) as u64;
        }
        let (index_buffer, index_buffer_alloc) = allocator.create_buffer(
            index_buffer_size as u64,
            vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST | extra_flags,
            gpu_allocator::MemoryLocation::GpuOnly,
        );

        uploader.enqueue_buffer_upload(vertex_buffer, 0, &mesh_data.vertices);

        let mut offset = 0u64;
        for sm in &mesh_data.submeshes {
            uploader.enqueue_buffer_upload(index_buffer, offset, &sm.faces);
            offset += (sm.faces.len() * size_of::<Face>()) as u64;
        }

        let mut submeshes = Vec::with_capacity(mesh_data.submeshes.len());
        let mut start_index = 0u32;
        for sm in &mesh_data.submeshes {
            submeshes.push((start_index, sm.faces.len() as u32 * 3));
            start_index += sm.faces.len() as u32 * 3;
        }

        let rtx_data = rtx_ext.map(|rtx_ext| {
            let (acc, acc_buf, acc_alloc) = uploader.enqueue_acc_struct_build(
                rtx_ext.clone(),
                vertex_buffer,
                index_buffer,
                &submeshes,
                mesh_data.vertices.len() as u32,
            );
            MeshRtxData {
                acc_struct: acc,
                buffer: acc_buf,
                buffer_alloc: acc_alloc,
                rtx_ext,
            }
        });

        Ok(Rc::new(Mesh {
            allocator,
            vertex_buffer,
            vertex_buffer_alloc,
            index_buffer,
            index_buffer_alloc,
            submeshes,

            rtx_data,
        }))
    }
}
