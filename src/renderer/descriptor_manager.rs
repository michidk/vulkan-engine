use std::{
    array,
    collections::{hash_map::DefaultHasher, BTreeMap},
    hash::{Hash, Hasher},
};

use ash::{
    version::DeviceV1_0,
    vk::{self, DescriptorSetAllocateInfo, DescriptorType, Handle},
    Device,
};

#[derive(Hash, Copy, Clone, PartialEq, Eq)]
pub enum DescriptorData {
    UniformBuffer {
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
        size: vk::DeviceSize
    }
}

struct DescriptorSetData {
    data_hash: u64,
    layout: vk::DescriptorSetLayout,
    frame_index: u16,
    set: vk::DescriptorSet,
}

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
                .descriptor_count(4096)
                .ty(vk::DescriptorType::UNIFORM_BUFFER)
                .build(),
            vk::DescriptorPoolSize::builder()
                .descriptor_count(4096)
                .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .build(),
        ];
        let poolInfo = vk::DescriptorPoolCreateInfo::builder()
            .max_sets(4096)
            .pool_sizes(&pool_sizes)
            .flags(vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET)
            .build();

        let pool = unsafe { device.create_descriptor_pool(&poolInfo, None) }?;

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
        let allocInfo = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(self.pool)
            .set_layouts(&[layout])
            .build();
        let newSet = unsafe { self.device.allocate_descriptor_sets(&allocInfo)?[0] };

        let mut bufferInfos = Vec::with_capacity(bindings.len());
        let mut imageInfos = Vec::with_capacity(bindings.len());
        let mut setWrites = Vec::with_capacity(bindings.len());
        for (index, b) in bindings.iter().enumerate() {
            match b {
                DescriptorData::UniformBuffer {
                    buffer,
                    offset,
                    size,
                } => {
                    bufferInfos.push(
                        vk::DescriptorBufferInfo::builder()
                            .buffer(*buffer)
                            .offset(*offset)
                            .range(*size)
                            .build(),
                    );
                    setWrites.push(
                        vk::WriteDescriptorSet::builder()
                            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                            .dst_binding(index as u32)
                            .dst_set(newSet)
                            .buffer_info(&[*bufferInfos.last().unwrap()])
                            .build(),
                    );
                }
                DescriptorData::StorageBuffer {
                    buffer, offset, size
                } => {
                    bufferInfos.push(
                        vk::DescriptorBufferInfo::builder()
                            .buffer(*buffer)
                            .offset(*offset)
                            .range(*size)
                            .build(),
                    );
                    setWrites.push(
                        vk::WriteDescriptorSet::builder()
                            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                            .dst_binding(index as u32)
                            .dst_set(newSet)
                            .buffer_info(&[*bufferInfos.last().unwrap()])
                            .build(),
                    );
                }
                DescriptorData::ImageSampler {
                    image,
                    layout,
                    sampler,
                } => {
                    imageInfos.push(
                        vk::DescriptorImageInfo::builder()
                            .image_view(*image)
                            .image_layout(*layout)
                            .sampler(*sampler)
                            .build(),
                    );
                    setWrites.push(
                        vk::WriteDescriptorSet::builder()
                            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                            .dst_binding(index as u32)
                            .dst_set(newSet)
                            .image_info(&[*imageInfos.last().unwrap()])
                            .build(),
                    );
                }
            }
        }
        unsafe { self.device.update_descriptor_sets(&setWrites, &[]) };

        self.frame_sets[self.frame_index as usize].push(hash);
        assert!(self
            .set_cache
            .insert(
                hash,
                DescriptorSetData {
                    data_hash: hash,
                    layout: layout,
                    frame_index: self.frame_index,
                    set: newSet,
                }
            )
            .is_none());

        return Ok(newSet);
    }

    pub fn destroy(&mut self) {
        if self.pool != vk::DescriptorPool::null() {
            unsafe { self.device.destroy_descriptor_pool(self.pool, None); }
            self.pool = vk::DescriptorPool::null();
        }
    }
}

impl<const HISTORY_SIZE: usize> Drop for DescriptorManager<HISTORY_SIZE> {
    fn drop(&mut self) {
        self.destroy();
    }
}
