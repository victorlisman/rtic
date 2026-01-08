use critical_section::CriticalSection;

/// Trait that must be implemented by the device interrupt enum for the R52 backend.
pub trait R52Interrupt {
    /// Pend the interrupt (used for dispatchers).
    fn pend(self);
}

/// Sets the given interrupt as pending.
#[inline]
pub fn pend<I>(interrupt: I)
where
    I: R52Interrupt,
{
    interrupt.pend();
}

/// Runs `f` at the given logical priority.
#[inline(always)]
pub fn run<F>(_: u8, f: F)
where
    F: FnOnce(),
{
    f();
}

/// Lock implementation using a global critical section.
#[inline(always)]
pub unsafe fn lock<T, R>(ptr: *mut T, _ceiling: u8, f: impl FnOnce(&mut T) -> R) -> R {
    critical_section::with(|_cs: CriticalSection<'_>| unsafe { f(&mut *ptr) })
}

/// Interrupt control for the R52 backend.
pub mod interrupt {
    /// Disable IRQs.
    #[inline]
    pub fn disable() {
        cortex_ar::interrupt::disable();
    }

    /// Enable IRQs.
    #[inline]
    pub fn enable() {
        // Safety: we only re-enable IRQs after RTIC init completes.
        unsafe { cortex_ar::interrupt::enable() };
    }
}

/// Stub peripheral struct for RTIC core access.
pub struct Peripherals;

impl Peripherals {
    /// Steal the core peripherals (no-op on R52 backend).
    pub fn steal() -> Self {
        Peripherals
    }
}
