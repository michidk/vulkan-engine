use std::{mem::size_of, rc::Rc};

use ash::vk;
use ve_format::mesh::{Face, MeshData, Vertex};

use crate::vulkan::uploader::Uploader;
pub struct Mesh {
    allocator: Rc<vk_mem::Allocator>,
    pub vertex_buffer: vk::Buffer,
    pub vertex_buffer_alloc: vk_mem::Allocation,
    pub index_buffer: vk::Buffer,
    pub index_buffer_alloc: vk_mem::Allocation,
    pub submeshes: Vec<(u32, u32)>,
}

impl Drop for Mesh {
    fn drop(&mut self) {
        self.allocator
            .destroy_buffer(self.vertex_buffer, &self.vertex_buffer_alloc);
        self.allocator
            .destroy_buffer(self.index_buffer, &self.index_buffer_alloc);
    }
}

impl Mesh {
    pub fn bake(
        mesh_data: MeshData,
        allocator: Rc<vk_mem::Allocator>,
        uploader: &mut Uploader
    ) -> Result<Rc<Mesh>, vk_mem::Error> {
        let vertex_buffer_size = mesh_data.vertices.len() * size_of::<Vertex>();
        let vertex_buffer_info = vk::BufferCreateInfo::builder()
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .size(vertex_buffer_size as u64)
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST)
            .build();
        let vertex_buffer_alloc_info = vk_mem::AllocationCreateInfo {
            usage: vk_mem::MemoryUsage::GpuOnly,
            ..Default::default()
        };
        let (vertex_buffer, vertex_buffer_alloc, _) =
            allocator.create_buffer(&vertex_buffer_info, &vertex_buffer_alloc_info)?;

        let mut index_buffer_size = 0u64;
        for sm in &mesh_data.submeshes {
            index_buffer_size += (sm.faces.len() * size_of::<Face>()) as u64;
        }
        let index_buffer_info = vk::BufferCreateInfo::builder()
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .size(index_buffer_size as u64)
            .usage(vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST)
            .build();
        let index_buffer_alloc_info = vk_mem::AllocationCreateInfo {
            usage: vk_mem::MemoryUsage::GpuOnly,
            ..Default::default()
        };
        let (index_buffer, index_buffer_alloc, _) =
            allocator.create_buffer(&index_buffer_info, &index_buffer_alloc_info)?;

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

        Ok(Rc::new(Mesh {
            allocator,
            vertex_buffer,
            vertex_buffer_alloc,
            index_buffer,
            index_buffer_alloc,
            submeshes,
        }))
    }
}
