use crate::gbdk_c;

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum InterruptKind {
    VBlank = 0x01,
    LCD = 0x02,
    Timer = 0x04,
    Serial = 0x08,
    Joypad = 0x10,
}

#[derive(Clone, Copy)]
pub struct Interrupt {
    pub kind: InterruptKind,
    pub handler: unsafe extern "C" fn(),
}

impl Interrupt {
    unsafe fn register(&self) {
        match self.kind {
            InterruptKind::VBlank => unsafe { gbdk_c::gb::gb::add_VBL(self.handler) },
            InterruptKind::LCD => unsafe { gbdk_c::gb::gb::add_LCD(self.handler) },
            InterruptKind::Timer => unsafe { gbdk_c::gb::gb::add_TIM(self.handler) },
            InterruptKind::Serial => unsafe { gbdk_c::gb::gb::add_SIO(self.handler) },
            InterruptKind::Joypad => unsafe { gbdk_c::gb::gb::add_JOY(self.handler) },
        }
    }
}

pub struct InterruptBuilder {
    int_flag: u8,
}

impl InterruptBuilder {
    pub fn new() -> Self {
        Self { int_flag: 0x0 }
    }

    pub unsafe fn register_irq(&mut self, int: Interrupt) -> &mut Self {
        int.register();
        self.int_flag |= int.kind as u8;
        self
    }

    pub unsafe fn enable(&self) {
        gbdk_c::gb::gb::set_interrupts(self.int_flag);
        gbdk_c::gb::gb::enable_interrupts();
    }

    pub unsafe fn disable(&self) {
        gbdk_c::gb::gb::disable_interrupts();
    }
}
