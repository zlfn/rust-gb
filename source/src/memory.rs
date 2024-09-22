/// Rust's memory safety rules assume your program operates in a linear, non-segmented memory
/// space. This means that Rust assumes there are no "holes" in the address space and writing to
/// any place in memory never has any affect on other parts of memory. Neither of these are true in
/// the Gameboy's memory space.
///
/// There are unused regions of memory address space that are unused. Writing to those addresses to
/// do nothing, and reading from those addresses will provide garbage data. This struct helps
/// ensure that you will never access these regions.
// TODO: Fill in the rest of the fields for the various regions of memory. The inner types should
// be as small as possible (ideally zero sized). For regions where only part of the register can be
// writen to (for example, 0xFF70, the WRAM selection register, only has three bytes that can be
// writen to), those types should avoid giving out a reference to their register(s). Rather, those
// types should act like `Cell`s
// NOTE: We want this type to be non exhaustive to force users to only construct it via
// `Self::new`.
#[non_exhaustive]
pub struct MemoryMap {
    /// The manager for the external RAM banks
    pub external_ram: Option<RamManager>,
    /// The manager for the work RAM banks
    pub work_ram_1: Option<RamManager>,
    pub work_ram_0: Option<RamManager>,
    /// The manager for the high RAM banks
    pub high_ram: Option<RamManager>
}

impl MemoryMap {
    /// This method should be called at the very start of your program in order to set up all the
    /// necessary parts of memory that need to be tracked and initialized.
    ///
    /// SAFETY:
    /// This function is the first thing you call in your program. 
    /// Any existing data might get over overwriten.
    /// In addition, there should be a guarantee that the area is not invaded by the stack or other dependencies.
    pub unsafe fn new(external_ram: Option<RamManager>, work_ram_1: Option<RamManager>, work_ram_0: Option<RamManager>, high_ram: Option<RamManager>) -> Self {
        Self {
            external_ram,
            work_ram_1,
            work_ram_0,
            high_ram,
        }
    }

    /// This function maps memory areas that are not used by other dependent tools 
    ///
    /// SAFETY:
    /// This function is the first thing you call in your program. Any existing data might get over
    /// overwriten.
    pub unsafe fn default() -> Self {
        Self {
            external_ram: Some(RamManager::new(Some(0x0000), 0xA000)),
            work_ram_1: None,
            work_ram_0: Some(RamManager::new(None, 0xC000)),
            high_ram: None,
        }
    }
}

// TODO: Make this type general enough to describe access to any series of RAM bank. Currently, it
// assumes you are accessing the external RAM banks
/// Manages access to a series of RAM banks that overlap in a memory space as well as the register
/// used to toggle between them.
#[non_exhaustive]
pub struct RamManager {
    bank_selection_addr: Option<u16>,
    bank_start_addr: u16,
}

impl RamManager {
    /// This method should only be called by `MemoryMap::new`.
    ///
    /// SAFETY:
    /// Same safety rules as `MemoryMap::new`
    unsafe fn new(bank_selection_addr: Option<u16>, bank_start_addr: u16) -> Self {
        RamManager {
            bank_selection_addr,
            bank_start_addr
        }
    }

    unsafe fn select_bank(&mut self, bank: u8) {
        // TODO: There is an unstable feature to provide `as_mut_unchecked`. This should probably
        // use that.
        match self.bank_selection_addr {
            Some(bank_selection_addr) => {
                *(bank_selection_addr as *mut u8).as_mut().unwrap() = bank;
            }
            None => {
                panic!("Trying bank switching to RAM that cannot be switched.")
            }
        }
    }

    unsafe fn read_bank(&self) -> u8 {
        // TODO: There is an unstable feature to provide `as_ref_unchecked`. This should probably
        // use that.
        *(self.bank_start_addr as *const u8).as_ref().unwrap()
    }

    /// This fetches the necessary memory bank.
    ///
    /// Note that this requires a mutable reference, so the bank can not change while the given
    /// `RamBank` is out.
    pub fn fetch_bank(&mut self, bank: u8) -> &mut RamBank {
        unsafe {
            self.select_bank(bank);
            (self.bank_start_addr as *mut RamBank).as_mut().unwrap()
        }
    }

    /// If we only need one piece of data (maybe because we're moving it), we can just return a
    /// reference to it and still get lifetime safety.
    ///
    /// Note: This can not be a `Deref` impl. We need this method to take a mutable reference to the
    /// `BankManager` even if we only need a shared reference to the data.
    fn fetch<T>(&mut self, ptr: &mut BankPtr<T>) -> &mut T {
        let bank = self.fetch_bank(ptr.bank_number);
        bank.fetch_mut(ptr)
    }
}

// I'm not sure what all this would hold... It depends on if this type manages how the bank's
// memory is allocated.
struct RamBank {
    bank_number: u8,
}

impl RamBank {
    // This can be a `Deref` impl. Also, we could provide an unchecked version and a version that
    // returns `None` if the banks mismatch.
    fn fetch<T>(&self, ptr: &BankPtr<T>) -> &T {
        assert_eq!(ptr.bank_number, self.bank_number);
        // SAFETY:
        // If we assume that the RamBank manages this space, the pointer should be aligned
        // correctly, at the right address, and contains the correct type.
        // Also, we are using the borrow checker to ensure that access to `ptr` maps onto access to
        // the type in memory.
        unsafe { ptr.ptr.as_ref().unwrap() }
    }

    fn fetch_mut<T>(&mut self, ptr: &mut BankPtr<T>) -> &mut T {
        assert_eq!(ptr.bank_number, self.bank_number);
        unsafe { ptr.ptr.as_mut().unwrap() }
    }
}

// This type is contructed by the type that manages the data in banks (probably `RamBank`).
// NOTE: We can not implement deref on this type.
// TODO: I think this would need to have a `Drop` impl...
struct BankPtr<T> {
    bank_number: u8,
    ptr: *mut T,
}
