#![no_std]

#[no_mangle]
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

pub mod lsh {
    #[no_mangle]
    pub fn lsh(left: u64, right: u64) -> u64 {
        left << right
    }
}
