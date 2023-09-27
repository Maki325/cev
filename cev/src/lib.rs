use std::alloc::{alloc, dealloc, Layout};
use std::marker::PhantomData;

pub trait Compress {
  fn size(&self) -> usize;
  fn compress(&self, ptr: *mut u8);
  fn uncompress(ptr: *const u8) -> Self;
  fn read_size(ptr: *const u8) -> usize;
}

macro_rules! impl_compression {
  ($($ty:ty, $size:expr);+) => {
    $(
      impl Compress for $ty {
        fn size(&self) -> usize {
          return $size;
        }
        fn compress(&self, ptr: *mut u8) {
          unsafe {
            *(ptr as *mut $ty) = *self;
          }
        }
        fn uncompress(ptr: *const u8) -> Self {
          unsafe { *(ptr as *const $ty) }
        }
        fn read_size(_ptr: *const u8) -> usize {
          return $size;
        }
      }
    )+
  };
}

impl_compression!(u8, 1; u16, 2; u32, 4; u64, 8; u128, 16);
impl_compression!(i8, 1; i16, 2; i32, 4; i64, 8; i128, 16);

pub struct Compressed<T: Compress> {
  _phantom: PhantomData<T>,
  pub data: *mut u8,
  len: usize,
  offset: usize,
  pub cap: usize,

  pub offsets: Vec<usize>,
}

const START_CAP: usize = 32;

impl<T: Compress> Compressed<T> {
  pub fn new() -> Self {
    return Self::with_capacity(START_CAP);
  }

  pub fn with_capacity(cap: usize) -> Self {
    let ptr = unsafe {
      let layout = Layout::from_size_align_unchecked(cap, std::mem::size_of::<T>());
      alloc(layout)
    };
    return Self {
      _phantom: PhantomData,
      data: ptr,
      len: 0,
      offset: 0,
      cap,
      offsets: Vec::with_capacity(cap),
    };
  }

  pub fn len(&self) -> usize {
    self.len
  }

  fn grow(&mut self) {
    let new_cap = self.cap * 2;
    let new_ptr = unsafe {
      let layout = Layout::from_size_align_unchecked(new_cap, std::mem::size_of::<T>());
      alloc(layout)
    };
    unsafe {
      std::ptr::copy_nonoverlapping(self.data, new_ptr, self.cap);
      let layout = Layout::from_size_align_unchecked(self.cap, std::mem::size_of::<T>());
      dealloc(self.data, layout);
    }
    self.data = new_ptr;
    self.cap = new_cap;
  }

  pub fn push(&mut self, val: T) {
    let size = val.size();
    if (self.offset + size) >= self.cap {
      self.grow();
    }
    unsafe {
      let ptr = self.data.add(self.offset);
      val.compress(ptr);
    }
    self.offsets.push(self.offset);
    self.offset += size;
    self.len += 1;
  }

  pub fn get(&self, idx: usize) -> Option<T> {
    if idx < self.len {
      return Some(T::uncompress(unsafe { self.data.add(self.offsets[idx]) }));
    } else {
      None
    }
  }

  /// Not checking the index is safe, leaving that to the callers
  fn replace(&mut self, idx: usize, old_size: usize, val: T) {
    let new_size = val.size();

    if new_size > old_size && self.offset + (new_size - old_size) >= self.cap {
      self.grow();
    }

    let ptr = unsafe { self.data.add(self.offsets[idx]) };

    // Move bytes so that we can fit the new value
    if new_size > old_size {
      if self.len - 1 != idx {
        let src = unsafe { ptr.add(old_size) };
        let dst = unsafe { ptr.add(new_size) };

        // We use `self.offset` and not `self.offsets[self.len - 1]`
        // Because the last offset in the list is the START of the last element
        // And we want the END of the last element
        let count = self.offset - (self.offsets[idx + 1]);
        unsafe {
          std::ptr::copy(src, dst, count);
        }

        let difference = new_size - old_size;

        self.offsets.iter_mut().enumerate().for_each(|(i, offset)| {
          if i > idx {
            *offset += difference;
          }
        });
        self.offset += difference;
      } else {
        unsafe {
          std::ptr::write_bytes(ptr, 0, old_size);
        }
      }
    }
    val.compress(ptr);

    // Move bytes so that there is no gap, and zero out the difference
    if new_size < old_size {
      if self.len - 1 != idx {
        let src = unsafe { ptr.add(old_size) };
        let dst = unsafe { ptr.add(new_size) };

        // We use `self.offset` and not `self.offsets[self.len - 1]`
        // Because the last offset in the list is the START of the last element
        // And we want the END of the last element
        let count = self.offset - (self.offsets[idx + 1]);
        let difference = old_size - new_size;

        unsafe {
          std::ptr::copy(src, dst, count);
          std::ptr::write_bytes(dst.add(count), 0, difference);
        }

        self.offsets.iter_mut().enumerate().for_each(|(i, offset)| {
          if i > idx {
            *offset -= difference;
          }
        });
        self.offset -= difference;
      } else {
        unsafe {
          std::ptr::write_bytes(ptr, 0, old_size);
        }
      }
    }
  }

  pub fn update<F: FnOnce(&T) -> T>(&mut self, idx: usize, f: F) {
    if idx >= self.len {
      return;
    }
    let ptr = unsafe { self.data.add(self.offsets[idx]) };
    let before = T::uncompress(ptr);
    let old_size = before.size();
    let after = f(&before);
    self.replace(idx, old_size, after);
  }
}

// https://stackoverflow.com/a/64996483/6105289
impl<T: Compress> Drop for Compressed<T> {
  fn drop(&mut self) {
    unsafe {
      dealloc(
        self.data as *mut u8,
        Layout::from_size_align_unchecked(self.len, std::mem::size_of::<T>()),
      )
    };
  }
}
