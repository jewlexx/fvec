#![allow(clippy::missing_safety_doc)]

use std::{
    fs::{File, OpenOptions},
    marker::PhantomData,
    ops::{Deref, DerefMut},
    path::Path,
    slice,
};

use memmap2::MmapMut;

pub struct FVec<T: Sized> {
    data_type: PhantomData<T>,
    map: MmapMut,
    capacity: usize,
    file: File,
    len: usize,
}

impl<T: Sized> FVec<T> {
    pub unsafe fn new() -> Self {
        // TODO: Maybe get a better file name
        Self::from_path("./filevec.bin")
    }

    pub unsafe fn from_path(path: impl AsRef<Path>) -> Self {
        const START_CAPACITY: usize = 8;
        let data_size = std::mem::size_of::<T>();

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)
            .unwrap();

        let len = {
            let len = file.metadata().unwrap().len();
            if len == 0 {
                START_CAPACITY as u64
            } else {
                len
            }
        };

        file.set_len(len * (data_size as u64)).unwrap();

        Self {
            data_type: PhantomData,
            map: MmapMut::map_mut(&file).unwrap(),
            capacity: START_CAPACITY,
            file,
            len: (len / (data_size as u64)) as usize,
        }
    }

    pub unsafe fn push(&mut self, data: T) {
        self.len += 1;
        if self.len > self.capacity {
            self.increase_file(self.capacity * 2);
        }

        self.as_slice_mut()[self.len - 1] = data;
    }

    pub unsafe fn as_slice<'a>(&self) -> &'a [T] {
        slice::from_raw_parts::<'a>(self.map.as_ptr().cast::<T>(), self.len)
    }

    pub unsafe fn as_slice_mut<'a>(&self) -> &'a mut [T] {
        slice::from_raw_parts_mut::<'a>(self.map.as_ptr().cast::<T>().cast_mut(), self.len)
    }

    fn increase_file(&mut self, new_capacity: usize) {
        let file_cap = new_capacity * std::mem::size_of::<T>();

        self.file.set_len(file_cap as u64).unwrap();
        self.capacity = new_capacity;
    }
}

impl<T> Deref for FVec<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe { self.as_slice() }
    }
}

impl<T> DerefMut for FVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.as_slice_mut() }
    }
}

impl<T> Default for FVec<T> {
    fn default() -> Self {
        unsafe { Self::new() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_data() {
        let mut vec = unsafe { FVec::<usize>::new() };
        for _ in 0..vec.capacity * 2 {
            unsafe { vec.push(usize::MAX) };
        }
        let slice = unsafe { vec.as_slice() };

        for i in slice.iter() {
            assert_eq!(*i, usize::MAX);
        }
    }

    #[test]
    fn test_mutate_slice() {
        let mut vec = unsafe { FVec::<usize>::new() };
        for _ in 0..16 {
            unsafe { vec.push(usize::MAX) };
        }
        let slice = unsafe { vec.as_slice_mut() };
        slice[9] = 5;

        drop(vec);

        let vec = unsafe { FVec::<usize>::new() };
        assert_eq!(vec.get(9), Some(&5));
    }

    #[test]
    fn test_mutate_vector() {
        unimplemented!()
    }
}
