use std::{mem::size_of, rc::Rc};

use ash::vk;
use gpu_allocator::{vulkan::Allocation, MemoryLocation};
use ve_format::mesh::{Face, MeshData, Vertex};

use crate::vulkan::{allocator::Allocator, error::GraphicsResult, uploader::Uploader};
pub struct Mesh {
    allocator: Rc<Allocator>,
    pub(crate) vertex_buffer: vk::Buffer,
    pub(crate) vertex_buffer_alloc: Allocation,
    pub(crate) index_buffer: vk::Buffer,
    pub(crate) index_buffer_alloc: Allocation,
    pub(crate) submeshes: Vec<(u32, u32)>,
}

impl Drop for Mesh {
    fn drop(&mut self) {
        self.allocator
            .destroy_buffer(self.vertex_buffer, self.vertex_buffer_alloc.clone());
        self.allocator
            .destroy_buffer(self.index_buffer, self.index_buffer_alloc.clone());
    }
}

impl Mesh {
    pub fn bake(
        mut mesh_data: MeshData,
        allocator: Rc<Allocator>,
        uploader: &mut Uploader,
        calculate_normals: bool,
    ) -> GraphicsResult<Rc<Mesh>> {
        if calculate_normals {
            for submesh in &mesh_data.submeshes {
                for face in &submesh.faces {
                    let a = mesh_data.vertices[face.indices[0] as usize];
                    let b = mesh_data.vertices[face.indices[1] as usize];
                    let c = mesh_data.vertices[face.indices[2] as usize];

                    let ab = b.position - a.position;
                    let ac = c.position - a.position;

                    let normal = ac.cross(ab).normalized();

                    mesh_data.vertices[face.indices[0] as usize].normal += normal;
                    mesh_data.vertices[face.indices[0] as usize]
                        .normal
                        .normalize();

                    mesh_data.vertices[face.indices[1] as usize].normal += normal;
                    mesh_data.vertices[face.indices[1] as usize]
                        .normal
                        .normalize();

                    mesh_data.vertices[face.indices[2] as usize].normal += normal;
                    mesh_data.vertices[face.indices[2] as usize]
                        .normal
                        .normalize();
                }
            }
        }

        let vertex_buffer_size = mesh_data.vertices.len() * size_of::<Vertex>();
        let (vertex_buffer, vertex_buffer_alloc) = allocator.create_buffer(
            vertex_buffer_size as u64,
            vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
            MemoryLocation::GpuOnly,
        )?;

        let mut index_buffer_size = 0u64;
        for sm in &mesh_data.submeshes {
            index_buffer_size += (sm.faces.len() * size_of::<Face>()) as u64;
        }
        let (index_buffer, index_buffer_alloc) = allocator.create_buffer(
            index_buffer_size,
            vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
            MemoryLocation::GpuOnly,
        )?;

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
