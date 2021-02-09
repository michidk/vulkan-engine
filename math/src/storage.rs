use std::fmt;

pub type DefaultStorage<T, const R: usize, const C: usize> = ArrayStorage<T, R, C>;

pub trait Allocator<T, const R: usize, const C: usize> {
    type Buffer: StorageMut<T, R, C>;

    unsafe fn allocate_unitialized() -> Self::Buffer;
    fn allocate_from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self::Buffer;
}

pub trait Storage<T, const R: usize, const C: usize> {
    fn get(&self, row_idx: usize, col_idx: usize) -> Option<&T>;
    unsafe fn get_unchecked(&self, row_idx: usize, col_idx: usize) -> &T;

    fn strides(&self) -> (usize, usize) {
        (R, C)
    }
}

pub trait StorageMut<T, const R: usize, const C: usize> {
    fn get_mut(&mut self, row_idx: usize, col_idx: usize) -> Option<&mut T>;
    unsafe fn get_unchecked_mut(&mut self, row_idx: usize, col_idx: usize) -> &mut T;
}

#[repr(C)]
pub struct ArrayStorage<T, const R: usize, const C: usize> {
    pub(crate) data: [[T; R]; C],
}

impl<T, const R: usize, const C: usize> Copy for ArrayStorage<T, R, C> where T: Copy {}

impl<T, const R: usize, const C: usize> Clone for ArrayStorage<T, R, C>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }
}

impl<T, const R: usize, const C: usize> fmt::Debug for ArrayStorage<T, R, C>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut dbg = f.debug_struct("ArrayStorage");

        for col in 0..C {
            let name = format!("c{}", col);
            dbg.field(name.as_str(), &self.data[col]);
        }

        dbg.finish()
    }
}

impl<T, const R: usize, const C: usize> PartialEq for ArrayStorage<T, R, C>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.data.eq(&other.data)
    }
}

impl<T, const R: usize, const C: usize> From<ArrayStorage<T, R, C>> for [[T; R]; C] {
    fn from(value: ArrayStorage<T, R, C>) -> Self {
        value.data
    }
}

impl<T, const R: usize> From<ArrayStorage<T, R, 1>> for [T; R]
where
    T: Copy,
{
    fn from(value: ArrayStorage<T, R, 1>) -> Self {
        value.data[0]
    }
}

impl<T, const R: usize, const C: usize> ArrayStorage<T, R, C> {
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.data.iter_mut().flat_map(|c| c.iter_mut())
    }
}

impl<T, const R: usize, const C: usize> Storage<T, R, C> for ArrayStorage<T, R, C> {
    fn get(&self, row_idx: usize, col_idx: usize) -> Option<&T> {
        self.data.get(col_idx)?.get(row_idx)
    }

    unsafe fn get_unchecked(&self, row_idx: usize, col_idx: usize) -> &T {
        self.data.get_unchecked(col_idx).get_unchecked(row_idx)
    }
}

impl<T, const R: usize, const C: usize> StorageMut<T, R, C> for ArrayStorage<T, R, C> {
    fn get_mut(&mut self, row_idx: usize, col_idx: usize) -> Option<&mut T> {
        self.data.get_mut(col_idx)?.get_mut(row_idx)
    }

    unsafe fn get_unchecked_mut(&mut self, row_idx: usize, col_idx: usize) -> &mut T {
        self.data
            .get_unchecked_mut(col_idx)
            .get_unchecked_mut(row_idx)
    }
}

pub struct DefaultAllocator;

impl<T, const R: usize, const C: usize> Allocator<T, R, C> for DefaultAllocator {
    type Buffer = ArrayStorage<T, R, C>;

    unsafe fn allocate_unitialized() -> Self::Buffer {
        std::mem::MaybeUninit::<Self::Buffer>::uninit().assume_init()
    }

    fn allocate_from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self::Buffer {
        let mut buffer = unsafe { Self::allocate_unitialized() };
        let mut count = 0;

        for (buffer, element) in buffer.iter_mut().zip(iter.into_iter()) {
            *buffer = element;
            count += 1;
        }

        assert_eq!(
            count,
            R * C,
            "Failed to allocate matrix: iterator has not enought items"
        );

        buffer
    }
}
