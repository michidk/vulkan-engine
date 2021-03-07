use std::{
    collections::{hash_map::DefaultHasher, BTreeMap},
    hash::{Hash, Hasher},
    slice,
};

use ash::{version::DeviceV1_0, vk};

#[allow(dead_code)]
#[derive(Hash, Copy, Clone, PartialEq, Eq)]
pub enum DescriptorData {
    UniformBuffer {
        buffer: vk::Buffer,
        offset: vk::DeviceSize,
        size: vk::DeviceSize,
    },
    DynamicUniformBuffer {
        buffer: vk::Buffer,
        offset: vk::DeviceSize,
        size: vk::DeviceSize,
    },
    ImageSampler {
        image: vk::ImageView,
        layout: vk::ImageLayout,
        sampler: vk::Sampler,
    },
    StorageBuffer {
        buffer: vk::Buffer,
        offset: vk::DeviceSize,
        size: vk::DeviceSize,
    },
    InputAttachment {
        image: vk::ImageView,
        layout: vk::ImageLayout,
    }
}

struct DescriptorSetData {
    data_hash: u64,
    frame_index: u16,
    set: vk::DescriptorSet,
}

/// This class automatically creates, caches and updates descriptor sets
/// The descriptor caching mechanism is losely inspired by the system the Granite Engine uses
/// (http://themaister.net/blog/2019/04/20/a-tour-of-granites-vulkan-backend-part-3/).
pub struct DescriptorManager<const HISTORY_SIZE: usize> {
    device: ash::Device,
    pool: vk::DescriptorPool,
    frame_index: u16,
    frame_sets: [Vec<u64>; HISTORY_SIZE],
    set_cache: BTreeMap<u64, DescriptorSetData>,
}

impl<const HISTORY_SIZE: usize> DescriptorManager<HISTORY_SIZE> {
    pub fn new(device: ash::Device) -> Result<DescriptorManager<HISTORY_SIZE>, vk::Result> {
        let pool_sizes = [
            vk::DescriptorPoolSize::builder()
                .descriptor_count(1024)
                .ty(vk::DescriptorType::UNIFORM_BUFFER)
                .build(),
            vk::DescriptorPoolSize::builder()
                .descriptor_count(1024)
                .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .build(),
            vk::DescriptorPoolSize::builder()
                .descriptor_count(1024)
                .ty(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
                .build(),
            vk::DescriptorPoolSize::builder()
                .descriptor_count(1024)
                .ty(vk::DescriptorType::STORAGE_BUFFER)
                .build(),
        ];
        let pool_info = vk::DescriptorPoolCreateInfo::builder()
            .max_sets(4096)
            .pool_sizes(&pool_sizes)
            .flags(vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET)
            .build();

        let pool = unsafe { device.create_descriptor_pool(&pool_info, None) }?;

        const VAL: Vec<u64> = Vec::new();

        Ok(DescriptorManager {
            device,
            pool,
            frame_index: 0,
            frame_sets: [VAL; HISTORY_SIZE],
            set_cache: BTreeMap::new(),
        })
    }

    pub fn next_frame(&mut self) {
        self.frame_index = (self.frame_index + 1) % HISTORY_SIZE as u16;

        // free old descriptor sets
        for s in &self.frame_sets[self.frame_index as usize] {
            let removed = self.set_cache.remove(s);
            if let Some(removed) = removed {
                unsafe { self.device.free_descriptor_sets(self.pool, &[removed.set]) };
            }
        }
        self.frame_sets[self.frame_index as usize].clear();
    }

    fn refresh_set(
        frame_index: u16,
        frame_sets: &mut [Vec<u64>; HISTORY_SIZE],
        set: &mut DescriptorSetData,
    ) {
        if set.frame_index == frame_index {
            return;
        }

        let vec = &mut frame_sets[set.frame_index as usize];
        let removed = vec.remove(vec.iter().position(|x| *x == set.data_hash).unwrap());

        frame_sets[frame_index as usize].push(removed);
        set.frame_index = frame_index;
    }

    pub fn get_descriptor_set(
        &mut self,
        layout: vk::DescriptorSetLayout,
        bindings: &[DescriptorData],
    ) -> Result<vk::DescriptorSet, vk::Result> {
        let mut hasher = DefaultHasher::new();
        for b in bindings {
            b.hash(&mut hasher);
        }
        let hash = hasher.finish();

        // step 1: Try to find active DescriptorSet with the same contents and layout
        if let Some(cached) = self.set_cache.get_mut(&hash) {
            let set = cached.set;
            Self::refresh_set(self.frame_index, &mut self.frame_sets, cached);
            return Ok(set);
        }

        // step 2: Try to allocate a new descriptor
        let alloc_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(self.pool)
            .set_layouts(slice::from_ref(&layout))
            .build();
        let new_set = unsafe { self.device.allocate_descriptor_sets(&alloc_info)?[0] };

        let mut buffer_infos = Vec::with_capacity(bindings.len());
        let mut image_infos = Vec::with_capacity(bindings.len());
        let mut set_writes = Vec::with_capacity(bindings.len());
        for (index, b) in bindings.iter().enumerate() {
            match b {
                DescriptorData::UniformBuffer {
                    buffer,
                    offset,
                    size,
                } => {
                    buffer_infos.push(
                        vk::DescriptorBufferInfo::builder()
                            .buffer(*buffer)
                            .offset(*offset)
                            .range(*size)
                            .build(),
                    );
                    set_writes.push(
                        vk::WriteDescriptorSet::builder()
                            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                            .dst_binding(index as u32)
                            .dst_set(new_set)
                            .buffer_info(slice::from_ref(buffer_infos.last().unwrap()))
                            .build(),
                    );
                }
                DescriptorData::DynamicUniformBuffer {
                    buffer,
                    offset,
                    size,
                } => {
                    buffer_infos.push(
                        vk::DescriptorBufferInfo::builder()
                            .buffer(*buffer)
                            .offset(*offset)
                            .range(*size)
                            .build(),
                    );
                    set_writes.push(
                        vk::WriteDescriptorSet::builder()
                            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
                            .dst_binding(index as u32)
                            .dst_set(new_set)
                            .buffer_info(slice::from_ref(buffer_infos.last().unwrap()))
                            .build(),
                    );
                }
                DescriptorData::StorageBuffer {
                    buffer,
                    offset,
                    size,
                } => {
                    buffer_infos.push(
                        vk::DescriptorBufferInfo::builder()
                            .buffer(*buffer)
                            .offset(*offset)
                            .range(*size)
                            .build(),
                    );
                    set_writes.push(
                        vk::WriteDescriptorSet::builder()
                            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                            .dst_binding(index as u32)
                            .dst_set(new_set)
                            .buffer_info(slice::from_ref(buffer_infos.last().unwrap()))
                            .build(),
                    );
                }
                DescriptorData::ImageSampler {
                    image,
                    layout,
                    sampler,
                } => {
                    image_infos.push(
                        vk::DescriptorImageInfo::builder()
                            .image_view(*image)
                            .image_layout(*layout)
                            .sampler(*sampler)
                            .build(),
                    );
                    set_writes.push(
                        vk::WriteDescriptorSet::builder()
                            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                            .dst_binding(index as u32)
                            .dst_set(new_set)
                            .image_info(slice::from_ref(image_infos.last().unwrap()))
                            .build(),
                    );
                }
                DescriptorData::InputAttachment { image, layout } => {
                    image_infos.push(
                        vk::DescriptorImageInfo::builder()
                            .image_view(*image)
                            .image_layout(*layout)
                            .build(),
                    );
                    set_writes.push(
                        vk::WriteDescriptorSet::builder()
                            .descriptor_type(vk::DescriptorType::INPUT_ATTACHMENT)
                            .dst_binding(index as u32)
                            .dst_set(new_set)
                            .image_info(slice::from_ref(image_infos.last().unwrap()))
                            .build(),
                    );
                }
            }
        }
        unsafe { self.device.update_descriptor_sets(&set_writes, &[]) };

        self.frame_sets[self.frame_index as usize].push(hash);
        assert!(self
            .set_cache
            .insert(
                hash,
                DescriptorSetData {
                    data_hash: hash,
                    frame_index: self.frame_index,
                    set: new_set,
                }
            )
            .is_none());

        Ok(new_set)
    }

    pub fn destroy(&mut self) {
        if self.pool != vk::DescriptorPool::null() {
            unsafe {
                self.device.destroy_descriptor_pool(self.pool, None);
            }
            self.pool = vk::DescriptorPool::null();
        }
    }
}

impl<const HISTORY_SIZE: usize> Drop for DescriptorManager<HISTORY_SIZE> {
    fn drop(&mut self) {
        self.destroy();
    }
}
