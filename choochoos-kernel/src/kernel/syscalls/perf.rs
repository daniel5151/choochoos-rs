//! Syscall handler implementations. See the [`Kernel`] docs.

use core::ptr;

use crate::kernel::Kernel;

/// Syscall handler implementations.
impl Kernel {
    pub fn syscall_perf(&mut self, perf_data: Option<ptr::NonNull<abi::PerfData>>) {
        if let Some(mut perf_data) = perf_data {
            let perf_data = unsafe { perf_data.as_mut() };

            perf_data.idle_time_pct = 0; // XXX: actually track idle time
        }
    }
}
