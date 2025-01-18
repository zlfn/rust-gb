use core::{mem::transmute, ops::BitOr};

use crate::gbdk_c;

pub struct VBlank {
    private: (),
}

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum Interrupt {
    VBlank = 0x01,
    LCD = 0x02,
    Timer = 0x04,
    Serial = 0x08,
    Joypad = 0x10,
}

impl Interrupt {
    pub unsafe fn add(self, int: unsafe extern "C" fn()) -> Self {
        let int = unsafe { transmute(int) };
        match self {
            Self::VBlank => unsafe { gbdk_c::gb::gb::add_VBL(int) },
            Self::LCD => unsafe { gbdk_c::gb::gb::add_LCD(int) },
            Self::Timer => unsafe { gbdk_c::gb::gb::add_TIM(int) },
            Self::Serial => unsafe { gbdk_c::gb::gb::add_SIO(int) },
            Self::Joypad => unsafe { gbdk_c::gb::gb::add_JOY(int) },
        }
        return self;
    }

    pub unsafe fn enable() {
        unsafe { gbdk_c::gb::gb::enable_interrupts() };
    }

    pub unsafe fn disable() {
        unsafe { gbdk_c::gb::gb::disable_interrupts() };
    }

    pub fn wait_vblank() -> VBlank {
        unsafe { gbdk_c::gb::gb::vsync() };
        return VBlank { private: () };
    }

    pub unsafe fn set(&self) {
        unsafe { gbdk_c::gb::gb::set_interrupts(*self as u8) };
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct InterruptSet(u8);

impl From<InterruptSet> for u8 {
    fn from(value: InterruptSet) -> Self {
        value.0
    }
}

impl InterruptSet {
    pub unsafe fn set(&self) {
        unsafe { gbdk_c::gb::gb::set_interrupts(u8::from(*self)) };
    }
}

impl BitOr<Self> for Interrupt {
    type Output = InterruptSet;
    fn bitor(self, rhs: Self) -> Self::Output {
        InterruptSet(self as u8 | rhs as u8)
    }
}

impl BitOr<InterruptSet> for Interrupt {
    type Output = InterruptSet;
    fn bitor(self, rhs: InterruptSet) -> Self::Output {
        InterruptSet(self as u8 | u8::from(rhs))
    }
}

impl BitOr<Interrupt> for InterruptSet {
    type Output = InterruptSet;
    fn bitor(self, rhs: Interrupt) -> Self::Output {
        InterruptSet(u8::from(self) | rhs as u8)
    }
}

impl BitOr<InterruptSet> for InterruptSet {
    type Output = InterruptSet;
    fn bitor(self, rhs: InterruptSet) -> Self::Output {
        InterruptSet(u8::from(self) | u8::from(rhs))
    }
}
