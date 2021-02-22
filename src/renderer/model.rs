use std::collections::HashMap;

use ash::{version::DeviceV1_0, vk};
use crystal::prelude::*;

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
pub struct VertexData {
    pub position: Vec3<f32>,
    pub normal: Unit<Vec3<f32>>,
}

impl VertexData {
    fn new(position: [f32; 3], normal: [f32; 3]) -> Self {
        Self {
            position: position.into(),
            normal: Unit::new_normalize(normal.into()),
        }
    }

    fn midpoint(a: &VertexData, b: &VertexData) -> Self {
        Self {
            position: Vec3::new(
                0.5 * (a.position.x() + b.position.x()),
                0.5 * (a.position.y() + b.position.y()),
                0.5 * (a.position.z() + b.position.z()),
            ),
            normal: Unit::new_normalize(Vec3::new(
                0.5 * (a.normal.x() + b.normal.x()),
                0.5 * (a.normal.y() + b.normal.y()),
                0.5 * (a.normal.z() + b.normal.z()),
            )),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct InstanceData {
    pub model_matrix: Mat4<f32>,
    pub inverse_model_matrix: Mat4<f32>,
    pub color: Color,
    pub metallic: f32,
    pub roughness: f32,
}

impl InstanceData {
    pub fn from_matrix_color_metallic_roughness(
        model_matrix: Mat4<f32>,
        color: Color,
        metallic: f32,
        roughness: f32,
    ) -> Self {
        Self {
            inverse_model_matrix: model_matrix.try_inverse().unwrap(),
            model_matrix,
            color,
            metallic,
            roughness,
        }
    }
}

pub type DefaultModel = Model<VertexData, InstanceData>;

impl DefaultModel {
    pub fn cube() -> Self {
        // lbf: left bottom front
        let lbf = VertexData::new([-1.0, 1.0, -1.0], [-1.0, 1.0, -1.0]);
        let lbb = VertexData::new([-1.0, 1.0, 1.0], [-1.0, 1.0, 1.0]);
        let ltf = VertexData::new([-1.0, -1.0, -1.0], [-1.0, -1.0, -1.0]);
        let ltb = VertexData::new([-1.0, -1.0, 1.0], [-1.0, -1.0, 1.0]);
        let rbf = VertexData::new([1.0, 1.0, -1.0], [1.0, 1.0, -1.0]);
        let rbb = VertexData::new([1.0, 1.0, 1.0], [1.0, 1.0, 1.0]);
        let rtf = VertexData::new([1.0, -1.0, -1.0], [1.0, -1.0, -1.0]);
        let rtb = VertexData::new([1.0, -1.0, 1.0], [1.0, -1.0, 1.0]);

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

    pub fn icosahedron() -> Self {
        let phi = (1.0 + 5.0_f32.sqrt()) / 2.0;

        let darkgreen_front_top = VertexData::new([phi, 1.0, 0.0], [phi, 1.0, 0.0]); //0
        let darkgreen_front_bottom = VertexData::new([phi, -1.0, 0.0], [phi, -1.0, 0.0]); //1
        let darkgreen_back_top = VertexData::new([-phi, 1.0, 0.0], [-phi, 1.0, 0.0]); //2
        let darkgreen_back_bottom = VertexData::new([-phi, -1.0, 0.0], [-phi, -1.0, 0.0]); //3
        let lightgreen_front_right = VertexData::new([1.0, 0.0, -phi], [1.0, 0.0, -phi]); //4
        let lightgreen_front_left = VertexData::new([-1.0, 0.0, -phi], [-1.0, 0.0, -phi]); //5
        let lightgreen_back_right = VertexData::new([1.0, 0.0, phi], [1.0, 0.0, phi]); //6
        let lightgreen_back_left = VertexData::new([-1.0, 0.0, phi], [-1.0, 0.0, phi]); //7
        let purple_top_left = VertexData::new([0.0, phi, -1.0], [0.0, phi, -1.0]); //8
        let purple_top_right = VertexData::new([0.0, phi, 1.0], [0.0, phi, 1.0]); //9
        let purple_bottom_left = VertexData::new([0.0, -phi, -1.0], [0.0, -phi, -1.0]); //10
        let purple_bottom_right = VertexData::new([0.0, -phi, 1.0], [0.0, -phi, 1.0]); //11

        Model {
            vertices: vec![
                darkgreen_front_top,
                darkgreen_front_bottom,
                darkgreen_back_top,
                darkgreen_back_bottom,
                lightgreen_front_right,
                lightgreen_front_left,
                lightgreen_back_right,
                lightgreen_back_left,
                purple_top_left,
                purple_top_right,
                purple_bottom_left,
                purple_bottom_right,
            ],
            indicies: vec![
                0, 9, 8, //
                0, 8, 4, //
                0, 4, 1, //
                0, 1, 6, //
                0, 6, 9, //
                8, 9, 2, //
                8, 2, 5, //
                8, 5, 4, //
                4, 5, 10, //
                4, 10, 1, //
                1, 10, 11, //
                1, 11, 6, //
                2, 3, 5, //
                2, 7, 3, //
                2, 9, 7, //
                5, 3, 10, //
                3, 11, 10, //
                3, 7, 11, //
                6, 7, 9, //
                6, 11, 7, //
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

    pub fn sphere(refinements: u32) -> Self {
        let mut ico = Self::icosahedron();
        for _ in 0..refinements {
            ico.refine();
        }
        for v in &mut ico.vertices {
            v.position = Unit::new_normalize(v.position).into_inner();
        }
        ico
    }

    pub fn refine(&mut self) {
        let mut new_indicies = Vec::new();
        let mut midpoints = HashMap::<(u32, u32), u32>::new();

        println!("{}", self.indicies.len());
        for triangle in self.indicies.chunks(3) {
            let a = triangle[0];
            let b = triangle[1];
            let c = triangle[2];

            let vertex_a = self.vertices[a as usize];
            let vertex_b = self.vertices[b as usize];
            let vertex_c = self.vertices[c as usize];

            let mab = if let Some(ab) = midpoints.get(&(a, b)) {
                *ab
            } else {
                let vertex_ab = VertexData::midpoint(&vertex_a, &vertex_b);
                let mab = self.vertices.len() as u32;
                self.vertices.push(vertex_ab);
                midpoints.insert((a, b), mab);
                midpoints.insert((b, a), mab);
                mab
            };

            let mbc = if let Some(bc) = midpoints.get(&(b, c)) {
                *bc
            } else {
                let vertex_bc = VertexData::midpoint(&vertex_b, &vertex_c);
                let mbc = self.vertices.len() as u32;
                self.vertices.push(vertex_bc);
                midpoints.insert((b, c), mbc);
                midpoints.insert((c, b), mbc);
                mbc
            };

            let mca = if let Some(ca) = midpoints.get(&(c, a)) {
                *ca
            } else {
                let vertex_ca = VertexData::midpoint(&vertex_c, &vertex_a);
                let mca = self.vertices.len() as u32;
                self.vertices.push(vertex_ca);
                midpoints.insert((c, a), mca);
                midpoints.insert((a, c), mca);
                mca
            };
            new_indicies.extend_from_slice(&[mca, a, mab, mab, b, mbc, mbc, c, mca, mab, mbc, mca]);
        }
        self.indicies = new_indicies;
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct TexturedVertexData {
    pub position: [f32; 3],
    pub texcoord: [f32; 2],
}

#[repr(C)]
pub struct TexturedInstanceData {
    pub modelmatrix: [[f32; 4]; 4],
    pub inverse_modelmatrix: [[f32; 4]; 4],
}

impl TexturedInstanceData {
    pub fn from_matrix(modelmatrix: Mat4<f32>) -> TexturedInstanceData {
        TexturedInstanceData {
            modelmatrix: modelmatrix.into(),
            inverse_modelmatrix: modelmatrix.try_inverse().unwrap().into(),
        }
    }
}

pub type TextureQuadModel = Model<TexturedVertexData, TexturedInstanceData>;

impl Model<TexturedVertexData, TexturedInstanceData> {
    pub fn quad() -> Self {
        let lb = TexturedVertexData {
            position: [-1.0, 1.0, 0.0],
            texcoord: [0.0, 1.0],
        }; //lb: left-bottom
        let lt = TexturedVertexData {
            position: [-1.0, -1.0, 0.0],
            texcoord: [0.0, 0.0],
        };
        let rb = TexturedVertexData {
            position: [1.0, 1.0, 0.0],
            texcoord: [1.0, 1.0],
        };
        let rt = TexturedVertexData {
            position: [1.0, -1.0, 0.0],
            texcoord: [1.0, 0.0],
        };
        Model {
            vertices: vec![lb, lt, rb, rt],
            indicies: vec![0, 2, 1, 1, 2, 3],
            handle_to_index: std::collections::HashMap::new(),
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
