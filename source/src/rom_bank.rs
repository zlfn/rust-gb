use core::marker::PhantomData;

extern "C" {
    /// Performs ROM switching.
    #[link_name="switch_rom"]
    fn switch_rom(rom_number: u8);
}

/// Game Boy has a limited ROM address, which means that to use enough space, 
/// functions and constants must be divided and stored in multiple ROM banks.
///
/// GBDK requires for one C file from one bank, and each bank calls each other externally. 
/// But if you split the file like that with Rust, you'll have more "unsafe" parts.
///
/// Therefore, in rust-gb, you will make one struct for one ROM bank.
/// static data will be managed with field, 
/// function will be managed with impl function, 
/// and bank switching will be managed by RomBankMap.
#[allow(private_bounds)]
pub struct RomBankMap<T: BankProvider + BanksTrait> {
    bank1: T::Bank1,
    bank2: T::Bank2,
    bank3: T::Bank3,
    bank4: T::Bank4,
    // TODO: You might think it's a little weird, but bank5 to bank64 has to be declared in same
    // way.
    // I don't think it's a good code, but I couldn't think of better method.
    bank64: T::Bank64,
}

#[allow(private_bounds)]
impl<T: BankProvider + BanksTrait> RomBankMap<T> {
    /// This method should be called at the very start of your program in order to set up all the
    /// datas in ROM that need to be tracked and initialized.
    ///
    /// SAFETY:
    /// If this function is called multiple times, banking may become ambiguous.
    pub unsafe fn new() -> Self {
        RomBankMap {
            bank1: T::Bank1::create_bank(),
            bank2: T::Bank2::create_bank(),
            bank3: T::Bank3::create_bank(),
            bank4: T::Bank4::create_bank(),
            //TODO:  bank 5 .. bank63
            bank64: T::Bank64::create_bank(),
        }
    }

    /// Because this consume its own mutable reference, the bank struct's reference can only 
    /// be accessed one at a time.
    pub fn switch_to_bank1(&mut self) -> &mut T::Bank1 {
        unsafe {switch_rom(1)};
        &mut self.bank1
    }
    pub fn switch_to_bank2(&mut self) -> &mut T::Bank2 {
        unsafe {switch_rom(2)};
        &mut self.bank2
    }
    pub fn switch_to_bank3(&mut self) -> &mut T::Bank3 {
        unsafe {switch_rom(3)};
        &mut self.bank3
    }
    pub fn switch_to_bank4(&mut self) -> &mut T::Bank4 {
        unsafe {switch_rom(4)};
        &mut self.bank4
    }
    //TODO: bank5 .. bank63
    pub fn switch_to_bank64(&mut self) -> &mut T::Bank64 {
        unsafe {switch_rom(64)};
        &mut self.bank64
    }
}

/// Implement BankProvider for Banks with your Bank structs
/// This trait is used because compiler need to know which bank structs are assigned to the bank
/// number at compilation time.
///
/// In addition, because we can't banking just by compiling Rust, the implementation of this trait 
/// will be parsed by `rust-gb` compiler to provide banking.
pub trait BankProvider {
    type Bank1: RomBank = DefaultBank;
    type Bank2: RomBank = DefaultBank;
    type Bank3: RomBank = DefaultBank;
    type Bank4: RomBank = DefaultBank;
    //TODO: bank5 .. bank63
    type Bank64: RomBank = DefaultBank;
}

/// Implement BankProvider for this.
#[derive(Clone, Copy)]
pub struct Banks<'a> {
    phantom: PhantomData<&'a ()>
}

/// It's a little trick that makes sure the user only creates RomBankMap with Banks.
trait BanksTrait {}
impl BanksTrait for Banks<'_> {}

/// All ROM banks need to implement this trait.
pub trait RomBank {
    fn create_bank() -> Self;
}

/// Default bank with nothing. By default, It's optimized and not compiled by Rust compiler.
pub struct DefaultBank {}
impl RomBank for DefaultBank {
    fn create_bank() -> Self {
        DefaultBank{}
    }
}

///A wrapper to ensure the lifetime of static data within RomBank's lifetime 
pub struct RomRef<'a, T> {
    static_ref: &'a T
}

impl<T> RomRef<'_, T> {
    pub fn new(reference: &'static T) -> Self {
        RomRef { static_ref: reference }
    }

    pub fn fetch(&self) -> &T {
        self.static_ref
    }
}
