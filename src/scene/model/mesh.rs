use std::{mem::size_of, rc::Rc};

use ash::vk;
use ve_format::mesh::{Face, MeshData, Vertex};

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
        allocator: Rc<vk_mem::Allocator>
    ) -> Result<Rc<Mesh>, vk_mem::Error> {
        let vertex_buffer_size = mesh_data.vertices.len() * size_of::<Vertex>();
        let vertex_buffer_info = vk::BufferCreateInfo::builder()
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .size(vertex_buffer_size as u64)
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
            .build();
        let vertex_buffer_alloc_info = vk_mem::AllocationCreateInfo {
            usage: vk_mem::MemoryUsage::CpuToGpu,
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
            .usage(vk::BufferUsageFlags::INDEX_BUFFER)
            .build();
        let index_buffer_alloc_info = vk_mem::AllocationCreateInfo {
            usage: vk_mem::MemoryUsage::CpuToGpu,
            ..Default::default()
        };
        let (index_buffer, index_buffer_alloc, _) =
            allocator.create_buffer(&index_buffer_info, &index_buffer_alloc_info)?;

        let map = allocator.map_memory(&vertex_buffer_alloc)? as *mut Vertex;
        unsafe {
            map.copy_from_nonoverlapping(mesh_data.vertices.as_ptr(), mesh_data.vertices.len());
        }
        allocator.unmap_memory(&vertex_buffer_alloc);

        let map = allocator.map_memory(&index_buffer_alloc)? as *mut Face;
        let mut offset = 0usize;
        for sm in &mesh_data.submeshes {
            unsafe {
                map.add(offset)
                    .copy_from_nonoverlapping(sm.faces.as_ptr(), sm.faces.len())
            };
            offset += sm.faces.len();
        }
        allocator.unmap_memory(&index_buffer_alloc);

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
