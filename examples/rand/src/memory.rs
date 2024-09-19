use core::{mem::size_of, ops::{Index, IndexMut}};

/// Enum that representing memory bank
/// Indicates all readable and writeable memory address areas within the Game Boy.
pub enum MemBank {
    /// Switchable bank from cartridge
    /// A000-BFFF
    /// bank: 0 ~ 15
    ExternalRam(u8),

    /// Non-switchable bank
    /// C000-CFFF
    WorkRam0,

    /// Switchable for CGB, Non-switchable for others
    /// Be careful because this area is shared with stack area
    /// D000-DFFF
    /// bank: 0 ~ 7
    WorkRam1(u8),

    /// Non-switchable on-board work ram
    /// FF80-FFFE
    HighRam
}

/// Array corresponding to the memory of each bank
pub struct MemArray<T> {
    bank: MemBank,
    size: usize,
    ptr: *mut T,
}

impl<T> MemArray<T> {
    /// Creates a MemArray from the pointer and size.
    /// This is unsafe because MemArray is not given exclusive access to that memory area.
    ///
    /// If the pointer's address is different from the bank specified, a panic occurs.
    pub unsafe fn from_ptr(ptr: *mut T, bank: MemBank, size: usize) -> Self {
        match bank {
            MemBank::ExternalRam(_) => {
                if ptr.addr() < 0xA000 || 0xBFFF < ptr.addr() {
                    panic!("Address not match for bank\0")
                }
            }
            MemBank::WorkRam0 => {
                if ptr.addr() < 0xC000 || 0xCFFF < ptr.addr() {
                    panic!("Address not match for bank\0")
                }
            }
            MemBank::WorkRam1(_) => {
                if ptr.addr() < 0xD000 || 0xDFFF < ptr.addr() {
                    panic!("Address not match for bank\0")
                }
            }
            MemBank::HighRam => {
                if ptr.addr() < 0xFF80 || 0xFFFE < ptr.addr() {
                    panic!("Address not match for bank\0")
                }
            }
        }

        MemArray {
            bank,
            size,
            ptr
        }
    }
}

impl<T: Sized + Copy> MemArray<T> {
    /// Fill the entire range of MemArray with a specific value.
    /// Note: Sometimes this is optimized with memset by compiler.
    pub fn fill(&self, data: T) {
        for i in 0..self.size {
            unsafe {
                *self.ptr.add(i * size_of::<T>()) = data;
            }
        }
    }
}

impl<T: LoadFromMem<T>> Index<usize> for MemArray<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        //TODO: handle banking
        if index >= self.size {
            panic!("index bound exceeded\0")
        }

        unsafe {
            ref_from_mem(self.ptr.add(index))
        }
    }
}

impl<T: LoadFromMem<T>> IndexMut<usize> for MemArray<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        //TODO: handle banking
        if index >= self.size {
            panic!("index_mut bound exceeded\0")
        }

        unsafe {
            mut_from_mem(self.ptr.add(index))
        }
    }
}

/// Type can be referenced from memory.
/// By default, it is implemented for all types that have a `Size` trait.
pub trait LoadFromMem<T: Sized> {
    /// Gets the reference from the raw pointer.
    unsafe fn ref_from_mem<'a>(ptr: *const T) -> &'a T {
        ptr.as_ref().unwrap()
    }

    /// Gets the mutable reference from the raw pointer.
    unsafe fn mut_from_mem<'a>(ptr: *mut T) -> &'a mut T {
        ptr.as_mut().unwrap()
    }
}

impl<T: Sized> LoadFromMem<T> for T {}

/// Gets the reference from the raw pointer.
pub unsafe fn ref_from_mem<'a, T: LoadFromMem<T>>(ptr: *const T) -> &'a T {
    T::ref_from_mem(ptr)
}

/// Gets the mutable reference from the raw pointer.
pub unsafe fn mut_from_mem<'a, T: LoadFromMem<T>>(ptr: *mut T) -> &'a mut T {
    T::mut_from_mem(ptr)
}
