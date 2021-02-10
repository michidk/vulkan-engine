use std::collections::HashMap;

use ash::{version::DeviceV1_0, vk};
use math::prelude::*;

use crate::color::Color;

use super::{buffer, RendererError};

pub struct Model<V, I> {
    vertices: Vec<V>,
    indicies: Vec<u32>,
    handle_to_index: HashMap<usize, usize>,
    handles: Vec<usize>,
    instances: Vec<I>,
    fist_invisible: usize,
    next_handle: usize,
    vertex_buffer: Option<buffer::BufferWrapper>,
    index_buffer: Option<buffer::BufferWrapper>,
    instance_buffer: Option<buffer::BufferWrapper>,
}

#[allow(dead_code)]
impl<V, I> Model<V, I> {
    fn get(&self, handle: usize) -> Option<&I> {
        self.instances.get(*self.handle_to_index.get(&handle)?)
    }

    pub fn get_mut(&mut self, handle: usize) -> Option<&mut I> {
        self.instances.get_mut(*self.handle_to_index.get(&handle)?)
    }

    fn swap_by_handle(&mut self, handle1: usize, handle2: usize) -> Result<(), RendererError> {
        if handle1 == handle2 {
            return Ok(());
        }

        if let (Some(&index1), Some(&index2)) = (
            self.handle_to_index.get(&handle1),
            self.handle_to_index.get(&handle2),
        ) {
            self.handles.swap(index1, index2);
            self.instances.swap(index1, index2);
            self.handle_to_index.insert(index1, handle2);
            self.handle_to_index.insert(index2, handle1);
            Ok(())
        } else {
            Err(RendererError::InvalidHandle)
        }
    }

    fn swap_by_index(&mut self, index1: usize, index2: usize) {
        if index1 == index2 {
            return;
        }
        let handle1 = self.handles[index1];
        let handle2 = self.handles[index2];
        self.handles.swap(index1, index2);
        self.instances.swap(index1, index2);
        self.handle_to_index.insert(index1, handle2);
        self.handle_to_index.insert(index2, handle1);
    }

    fn is_visible(&self, handle: usize) -> Result<bool, RendererError> {
        Ok(self
            .handle_to_index
            .get(&handle)
            .ok_or(RendererError::InvalidHandle)?
            < &self.fist_invisible)
    }

    fn make_visible(&mut self, handle: usize) -> Result<(), RendererError> {
        if let Some(&index) = self.handle_to_index.get(&handle) {
            // if already visible to nothing
            if index < self.fist_invisible {
                return Ok(());
            }
            // else: move to position first_invisible and increase value of first_invisible
            self.swap_by_index(index, self.fist_invisible);
            self.fist_invisible += 1;
            Ok(())
        } else {
            Err(RendererError::InvalidHandle)
        }
    }

    fn make_invisible(&mut self, handle: usize) -> Result<(), RendererError> {
        if let Some(&index) = self.handle_to_index.get(&handle) {
            // if already invisible to nothing
            if index >= self.fist_invisible {
                return Ok(());
            }
            // else: move to position first_invisible and increase value of first_invisible
            self.swap_by_index(index, self.fist_invisible - 1);
            self.fist_invisible -= 1;
            Ok(())
        } else {
            Err(RendererError::InvalidHandle)
        }
    }

    fn insert(&mut self, element: I) -> usize {
        let handle = self.next_handle;
        self.next_handle += 1;
        let index = self.instances.len();
        self.instances.push(element);
        self.handles.push(handle);
        self.handle_to_index.insert(handle, index);
        handle
    }

    pub fn insert_visibly(&mut self, element: I) -> usize {
        let new_handle = self.insert(element);
        self.make_visible(new_handle)
            .expect("Failed to make newly inserted handle visible");
        new_handle
    }

    fn remove(&mut self, handle: usize) -> Result<I, RendererError> {
        if let Some(&index) = self.handle_to_index.get(&handle) {
            if index < self.fist_invisible {
                self.swap_by_index(index, self.fist_invisible - 1);
                self.fist_invisible -= 1;
            }
            self.swap_by_index(self.fist_invisible, self.instances.len() - 1);
            self.handles.pop();
            self.handle_to_index.remove(&handle);
            Ok(self.instances.pop().expect("Failed to pop instance"))
        } else {
            Err(RendererError::InvalidHandle)
        }
    }

    pub fn update_vertex_buffer(
        &mut self,
        allocator: &vk_mem::Allocator,
    ) -> Result<(), vk_mem::error::Error> {
        if let Some(buffer) = &mut self.vertex_buffer {
            buffer.fill(allocator, &self.vertices)?;
            Ok(())
        } else {
            let bytes = (self.vertices.len() * std::mem::size_of::<V>()) as u64;
            let mut buffer = buffer::BufferWrapper::new(
                &allocator,
                bytes,
                vk::BufferUsageFlags::VERTEX_BUFFER,
                vk_mem::MemoryUsage::CpuToGpu,
            )?;
            buffer.fill(allocator, &self.vertices)?;
            self.vertex_buffer = Some(buffer);
            Ok(())
        }
    }

    pub fn update_index_buffer(
        &mut self,
        allocator: &vk_mem::Allocator,
    ) -> Result<(), vk_mem::error::Error> {
        if let Some(buffer) = &mut self.index_buffer {
            buffer.fill(allocator, &self.indicies)?;
            Ok(())
        } else {
            let bytes = (self.indicies.len() * std::mem::size_of::<u32>()) as u64;
            let mut buffer = buffer::BufferWrapper::new(
                &allocator,
                bytes,
                vk::BufferUsageFlags::INDEX_BUFFER,
                vk_mem::MemoryUsage::CpuToGpu,
            )?;
            buffer.fill(allocator, &self.indicies)?;
            self.index_buffer = Some(buffer);
            Ok(())
        }
    }

    pub fn update_instance_buffer(
        &mut self,
        allocator: &vk_mem::Allocator,
    ) -> Result<(), vk_mem::error::Error> {
        if let Some(buffer) = &mut self.instance_buffer {
            buffer.fill(allocator, &self.instances[0..self.fist_invisible])?;
            Ok(())
        } else {
            let bytes = (self.fist_invisible * std::mem::size_of::<I>()) as u64;
            let mut buffer = buffer::BufferWrapper::new(
                &allocator,
                bytes,
                vk::BufferUsageFlags::VERTEX_BUFFER,
                vk_mem::MemoryUsage::CpuToGpu,
            )?;
            buffer.fill(allocator, &self.instances[0..self.fist_invisible])?;
            self.instance_buffer = Some(buffer);
            Ok(())
        }
    }

    pub fn draw(&self, logical_device: &ash::Device, command_buffer: vk::CommandBuffer) {
        if let Some(vertex_buffer) = &self.vertex_buffer {
            if let Some(index_buffer) = &self.index_buffer {
                if let Some(instance_buffer) = &self.instance_buffer {
                    if self.fist_invisible > 0 {
                        unsafe {
                            logical_device.cmd_bind_index_buffer(
                                command_buffer,
                                index_buffer.buffer,
                                0,
                                vk::IndexType::UINT32,
                            );
                            logical_device.cmd_bind_vertex_buffers(
                                command_buffer,
                                0,
                                &[vertex_buffer.buffer],
                                &[0],
                            );
                            logical_device.cmd_bind_vertex_buffers(
                                command_buffer,
                                1,
                                &[instance_buffer.buffer],
                                &[0],
                            );
                            logical_device.cmd_draw_indexed(
                                command_buffer,
                                self.indicies.len() as u32,
                                self.fist_invisible as u32,
                                0,
                                0,
                                0,
                            );
                        }
                    }
                }
            }
        }
    }

    pub fn cleanup(&mut self, allocator: &vk_mem::Allocator) {
        if let Some(buffer) = &mut self.vertex_buffer {
            buffer.cleanup(allocator)
        }

        if let Some(buffer) = &mut self.index_buffer {
            buffer.cleanup(allocator)
        }

        if let Some(buffer) = &mut self.instance_buffer {
            buffer.cleanup(allocator)
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct InstanceData {
    pub position: Mat4<f32>,
    pub color: Color,
}

pub type DefaultModel = Model<Vec3<f32>, InstanceData>;

impl DefaultModel {
    pub fn cube() -> Self {
        // lbf: left bottom front
        let lbf = Vec3::new(-1.0, 1.0, -1.0);
        let lbb = Vec3::new(-1.0, 1.0, 1.0);
        let ltf = Vec3::new(-1.0, -1.0, -1.0);
        let ltb = Vec3::new(-1.0, -1.0, 1.0);
        let rbf = Vec3::new(1.0, 1.0, -1.0);
        let rbb = Vec3::new(1.0, 1.0, 1.0);
        let rtf = Vec3::new(1.0, -1.0, -1.0);
        let rtb = Vec3::new(1.0, -1.0, 1.0);

        Model {
            vertices: vec![lbf, lbb, ltf, ltb, rbf, rbb, rtf, rtb],
            indicies: vec![
                0, 1, 5, 0, 5, 4, //bottom
                2, 7, 3, 2, 6, 7, //top
                0, 6, 2, 0, 4, 6, //front
                1, 3, 7, 1, 7, 5, //back
                0, 2, 1, 1, 2, 3, //left
                4, 5, 6, 5, 7, 6, //right
            ],
            handle_to_index: HashMap::new(),
            handles: Vec::new(),
            instances: Vec::new(),
            fist_invisible: 0,
            next_handle: 0,
            vertex_buffer: None,
            index_buffer: None,
            instance_buffer: None,
        }
    }
}
