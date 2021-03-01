use std::{mem::size_of, rc::Rc};

use ash::vk;
use crystal::prelude::*;

pub struct Mesh {
    allocator: Rc<vk_mem::Allocator>,
    pub vertex_buffer: vk::Buffer,
    pub vertex_buffer_alloc: vk_mem::Allocation,
    pub instance_buffer: vk::Buffer,
    pub instance_buffer_alloc: vk_mem::Allocation,
    pub index_buffer: vk::Buffer,
    pub index_buffer_alloc: vk_mem::Allocation,
    pub submeshes: Vec<(u32, u32)>,
}

impl Drop for Mesh {
    fn drop(&mut self) {
        self.allocator
            .destroy_buffer(self.vertex_buffer, &self.vertex_buffer_alloc);
        self.allocator
            .destroy_buffer(self.instance_buffer, &self.instance_buffer_alloc);
        self.allocator
            .destroy_buffer(self.index_buffer, &self.index_buffer_alloc);
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Vertex {
    pub position: Vec3<f32>,
    pub color: Vec3<f32>,
    pub normal: Vec3<f32>,
    pub uv: Vec2<f32>,
}
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Submesh {
    pub faces: Vec<Face>,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Face {
    pub indices: [u32; 3],
}

pub struct MeshData {
    pub vertices: Vec<Vertex>,
    pub submeshes: Vec<Submesh>,
}

impl MeshData {
    pub fn bake(
        &self,
        allocator: Rc<vk_mem::Allocator>,
        transform: Mat4<f32>,
        inv_transform: Mat4<f32>,
    ) -> Result<Rc<Mesh>, vk_mem::Error> {
        let vertex_buffer_size = self.vertices.len() * size_of::<Vertex>();
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

        let instance_buffer_size = size_of::<Mat4<f32>>() * 2;
        let instance_buffer_info = vk::BufferCreateInfo::builder()
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .size(instance_buffer_size as u64)
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
            .build();
        let instance_buffer_alloc_info = vk_mem::AllocationCreateInfo {
            usage: vk_mem::MemoryUsage::CpuToGpu,
            ..Default::default()
        };
        let (instance_buffer, instance_buffer_alloc, _) =
            allocator.create_buffer(&instance_buffer_info, &instance_buffer_alloc_info)?;

        let mut index_buffer_size = 0u64;
        for sm in &self.submeshes {
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
            map.copy_from_nonoverlapping(self.vertices.as_ptr(), self.vertices.len());
        }
        allocator.unmap_memory(&vertex_buffer_alloc);

        let map = allocator.map_memory(&instance_buffer_alloc)? as *mut Mat4<f32>;
        unsafe {
            map.copy_from_nonoverlapping(&transform, 1);
            map.add(1).copy_from_nonoverlapping(&inv_transform, 1);
        }
        allocator.unmap_memory(&instance_buffer_alloc);

        let map = allocator.map_memory(&index_buffer_alloc)? as *mut Face;
        let mut offset = 0usize;
        for sm in &self.submeshes {
            unsafe {
                map.add(offset)
                    .copy_from_nonoverlapping(sm.faces.as_ptr(), sm.faces.len())
            };
            offset += sm.faces.len();
        }
        allocator.unmap_memory(&index_buffer_alloc);

        let mut submeshes = Vec::with_capacity(self.submeshes.len());
        let mut start_index = 0u32;
        for sm in &self.submeshes {
            submeshes.push((start_index, sm.faces.len() as u32 * 3));
            start_index += sm.faces.len() as u32 * 3;
        }

        Ok(Rc::new(Mesh {
            allocator,
            vertex_buffer,
            vertex_buffer_alloc,
            instance_buffer,
            instance_buffer_alloc,
            index_buffer,
            index_buffer_alloc,
            submeshes,
        }))
    }
}
