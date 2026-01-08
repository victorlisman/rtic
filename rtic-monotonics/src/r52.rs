// MINIMAL monotonic implementation for r52 port

pub mod prelude {
    #[cfg(feature = "r52_virtual_timer")]
    pub use create::r52_virtual_timer;

    pub use crate::Monotonic;
    pub use fugit::{self, ExtU64, ExtU64Ceil};
}

use portable_atomic::{AtomicU64, Ordering};
use rtic_time::{
    half_period_counter::calculate_now,
    timer_queue::{TimerQueue, TimerQueueBackend},
};

//pac ???

mod _generated {
    #![allow(dead_code)]
    #![allow(unused_imports)]
    #![allow(non_snake_case)]

    include!(concat!(env!("OUT_DIR"), "/_generated.rs"));
}

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

/// VIRTUAL TIMER based monotonic 
#[cfg(feature = "r52_virtual_timer")]
#[macro_export]
macro_rules! r52_virtual_timer {
    ($name:ident, $tick_rate_hz:expr) => {
        $crate::__internal_create_r52_timer_struct!($name, VirtualTimerBackend, PLACE, $tick_rate_hz);
    }
}

macro_rules! make_timer {
    ($backend_name:ident, $timer:ident, $bits:ident, $overflow:ident, $tq:ident$(, doc: ($($doc:tt)*))?) => {
        $(
            #[cfg_attr(docsrs, doc(cfg($($doc)*)))]
        )?

        pub struct $backend_name;

        //pac??

        static $overflow: AtomicU64 = AtomicU64::new(0);
        static $tq: TimerQueue<$backend_name> = TimerQueue::new();

        impl $backend_name {
            // TODO: Timer impl

            pub fn _start() {
                unimplemented!();
            }


        }

        impl TimerQueueBackend for $backend_name {
            // TODO

            fn now() {
                unipmlemented!();
            }

            fn set_compare() {
                unipmlemented!();
            }

            fn clear_compare_flag() {
                unipmlemented!();
            }

            fn pend_interrupt() {
                unipmlemented!();
            }

            fn enable_timer() {
                unipmlemented!();
            }

            fn disable_timer() {
                unipmlemented!();
            }

            fn on_interrupt() {
                unipmlemented!();
            }
            
            fn timer_queue() {
                unimplemented!();
            }
        }
    }
}

#[cfg(feature = "r52_virtual_timer")]
make_timer!(VirtualTimerBackend, PLACE, u32, VIRTUAL_TIMER_OVERFLOWS, VIRTUAL_TIMER_TQ);