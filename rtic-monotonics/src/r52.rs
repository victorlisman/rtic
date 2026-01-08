//! [`Monotonic`](rtic_time::Monotonic) implementation for Cortex-R52 virtual timer.

/// Common definitions and traits for using the R52 monotonic.
pub mod prelude {
    #[cfg(feature = "r52_virtual_timer")]
    pub use crate::r52_virtual_timer;

    pub use crate::Monotonic;
    pub use fugit::{self, ExtU64, ExtU64Ceil};
}

#[cfg(feature = "r52_virtual_timer")]
use aarch32_cpu::generic_timer::{El1VirtualTimer, GenericTimer};
use portable_atomic::{AtomicU64, Ordering};
use rtic_time::timer_queue::{TimerQueue, TimerQueueBackend};

//pac ???

#[doc(hidden)]
#[macro_export]
macro_rules! __internal_create_r52_timer_interrupt {
    ($mono_backend:ident, $interrupt_name:ident) => {
        #[no_mangle]
        #[allow(non_snake_case)]
        unsafe extern "C" fn $interrupt_name() {
            use $crate::TimerQueueBackend;
            $crate::r52::$mono_backend::timer_queue().on_monotonic_interrupt();
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __internal_create_r52_timer_struct {
    ($name:ident, $mono_backend:ident, $timer:ident, $tick_rate_hz:expr) => {
        /// A `Monotonic` based on the Cortex-R52 virtual timer.
        pub struct $name;

        impl $name {
            pub fn start(tim_clock_hz: u32) {
                $crate::__internal_create_r52_timer_interrupt!($mono_backend, $timer);
                $crate::r52::$mono_backend::_start(tim_clock_hz, $tick_rate_hz);
            }
        }

        impl $crate::TimerQueueBasedMonotonic for $name {
            type Backend = $crate::r52::$mono_backend;
            type Instant = $crate::fugit::Instant<
                <Self::Backend as $crate::TimerQueueBackend>::Ticks,
                1,
                { $tick_rate_hz },
            >;
            type Duration = $crate::fugit::Duration<
                <Self::Backend as $crate::TimerQueueBackend>::Ticks,
                1,
                { $tick_rate_hz },
            >;
        }

        $crate::rtic_time::impl_embedded_hal_delay_fugit!($name);
        $crate::rtic_time::impl_embedded_hal_async_delay_fugit!($name);
    };
}

/// Create a virtual timer based monotonic and register the interrupt for it.
///
/// # Arguments
///
/// * `name` - The name that the monotonic type will have.
/// * `interrupt` - The interrupt symbol name used by the application.
/// * `tick_rate_hz` - The tick rate of the timer.
#[cfg(feature = "r52_virtual_timer")]
#[macro_export]
macro_rules! r52_virtual_timer {
    ($name:ident, $interrupt:ident, $tick_rate_hz:expr) => {
        $crate::__internal_create_r52_timer_struct!(
            $name,
            VirtualTimerBackend,
            $interrupt,
            $tick_rate_hz
        );
    };
    ($name:ident, $tick_rate_hz:expr) => {
        $crate::r52_virtual_timer!($name, VirtualTimer, $tick_rate_hz);
    };
}

macro_rules! make_timer {
    ($backend_name:ident, $timer:ident, $bits:ident, $overflow:ident, $tq:ident$(, doc: ($($doc:tt)*))?) => {
        $(
            #[cfg_attr(docsrs, doc(cfg($($doc)*)))]
        )?

        /// struct
        pub struct $backend_name;

        //pac??

        static $overflow: AtomicU64 = AtomicU64::new(0);
        static $tq: TimerQueue<$backend_name> = TimerQueue::new();
        static TICKS_PER_TICK: AtomicU64 = AtomicU64::new(0);

        /// RTIC backend
        impl $backend_name {
            /// start
            pub fn _start(tim_clock_hz: u32, tick_rate_hz: u32) {
                let timer_hz = if tim_clock_hz == 0 {
                    unsafe { El1VirtualTimer::new().frequency_hz() as u32 }
                } else {
                    tim_clock_hz
                };
                assert!(tick_rate_hz > 0, "tick rate must be non-zero");
                assert!(
                    (timer_hz % tick_rate_hz) == 0,
                    "tick rate must divide timer frequency"
                );
                let ticks_per_tick = (timer_hz / tick_rate_hz) as u64;
                assert!(ticks_per_tick > 0, "invalid tick scale");
                TICKS_PER_TICK.store(ticks_per_tick, Ordering::SeqCst);

                $tq.initialize(Self {});
                $overflow.store(0, Ordering::SeqCst);

                let mut vt = unsafe { El1VirtualTimer::new() };
                vt.enable(true);
                vt.interrupt_mask(false);
                let now = vt.counter();
                vt.counter_compare_set(now.wrapping_add(ticks_per_tick));
            }
        }

        impl TimerQueueBackend for $backend_name {
            type Ticks = u64;

            fn now() -> Self::Ticks {
                let ticks_per_tick = TICKS_PER_TICK.load(Ordering::Relaxed);
                assert!(ticks_per_tick > 0, "monotonic not started");
                let raw = unsafe { El1VirtualTimer::new().counter() };
                raw / ticks_per_tick
            }

            fn set_compare(instant: Self::Ticks) {
                let ticks_per_tick = TICKS_PER_TICK.load(Ordering::Relaxed);
                assert!(ticks_per_tick > 0, "monotonic not started");
                let raw_now = unsafe { El1VirtualTimer::new().counter() };
                let mut raw = instant.wrapping_mul(ticks_per_tick);
                if raw <= raw_now {
                    raw = raw_now.wrapping_add(ticks_per_tick);
                }
                unsafe { El1VirtualTimer::new().counter_compare_set(raw) };
            }

            fn clear_compare_flag() {
                let ticks_per_tick = TICKS_PER_TICK.load(Ordering::Relaxed);
                if ticks_per_tick == 0 {
                    return;
                }
                let now = unsafe { El1VirtualTimer::new().counter() };
                unsafe {
                    El1VirtualTimer::new().counter_compare_set(now.wrapping_add(ticks_per_tick));
                }
            }

            fn pend_interrupt() {
                let mut vt = unsafe { El1VirtualTimer::new() };
                vt.enable(true);
                vt.interrupt_mask(false);
                let now = vt.counter();
                vt.counter_compare_set(now.wrapping_add(1));
            }

            fn enable_timer() {
                let mut vt = unsafe { El1VirtualTimer::new() };
                vt.enable(true);
                vt.interrupt_mask(false);
            }

            fn disable_timer() {
                let mut vt = unsafe { El1VirtualTimer::new() };
                vt.interrupt_mask(true);
            }

            fn on_interrupt() {}

            fn timer_queue() -> &'static TimerQueue<$backend_name> {
                &$tq
            }
        }
    }
}

#[cfg(feature = "r52_virtual_timer")]
make_timer!(
    VirtualTimerBackend,
    PLACE,
    u32,
    VIRTUAL_TIMER_OVERFLOWS,
    VIRTUAL_TIMER_TQ
);
