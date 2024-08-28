use ballast_hpc_pac::clint::{HartId, CLINT};
use ballast_hpc_pac::hpc::CLINT;
use hpc_hal::slic::*;
use riscv_slic;
#[cfg(all(feature = "riscv-cva6", not(feature = "riscv-cva6-backend")))]
compile_error!("Building for the CVA6, but 'riscv-cva6-backend not selected'");

#[inline(always)]
pub fn run<F>(priority: u8, f: F)
where
    F: FnOnce(),
{
    if priority == 1 {
        //if priority is 1, priority thresh should be 1

        f();
        unsafe { riscv_slic::set_threshold(0x1) }
    } else {
        //read current thresh
        let initial = riscv_slic::get_threshold();
        f();
        //write back old thresh
        unsafe {
            riscv_slic::set_threshold(initial);
        }
    }
}

/// Lock implementation using threshold and global Critical Section (CS)
///
/// # Safety
///
/// The system ceiling is raised from current to ceiling
/// by either
/// - raising the threshold to the ceiling value, or
/// - disable all interrupts in case we want to
///   mask interrupts with maximum priority
///
/// Dereferencing a raw pointer inside CS
///
/// The priority.set/priority.get can safely be outside the CS
/// as being a context local cell (not affected by preemptions).
/// It is merely used in order to omit masking in case current
/// priority is current priority >= ceiling.
#[inline(always)]
pub unsafe fn lock<T, R>(ptr: *mut T, ceiling: u8, f: impl FnOnce(&mut T) -> R) -> R {
    if ceiling == (15) {
        //turn off interrupts completely, were at max prio
        let r = critical_section::with(|_| f(&mut *ptr));
        r
    } else {
        let current = riscv_slic::get_threshold();

        unsafe { riscv_slic::set_threshold(ceiling) }
        let r = f(&mut *ptr);
        unsafe { riscv_slic::set_threshold(current) }
        r
    }
}

/// Sets the given software interrupt as pending
#[inline(always)]
pub fn pend(int: Interrupt) {
    unsafe {
        match int {
            SoftwareInterrupts::Soft1 => riscv_slic::pend(SoftwareInterrupts::Soft1),
            SoftwareInterrupts::Soft2 => riscv_slic::pend(SoftwareInterrupts::Soft2),
            SoftwareInterrupts::Soft3 => riscv_slic::pend(SoftwareInterrupts::Soft3),
            _ => panic!("Unsupported software interrupt"), //should never happen, checked at compile time
        }
    }
}

// Sets the given software interrupt as not pending
pub fn unpend(int: Interrupt) {
    unsafe {
        match int {
            SoftwareInterrupts::Soft1 => riscv_slic::unpend(SoftwareInterrupts::Soft1),
            SoftwareInterrupts::Soft2 => riscv_slic::unpend(SoftwareInterrupts::Soft2),
            SoftwareInterrupts::Soft3 => riscv_slic::unpend(SoftwareInterrupts::Soft3),
            _ => panic!("Unsupported software interrupt"),
        }
    }
}

pub fn enable(int: Interrupt, prio: u8, cpu_int_id: u8) {
    unsafe {
        CLINT::mswi_enable();
        riscv_slic::set_interrupts(); //enables software interrupts in mie
        riscv_slic::enable(); //enables global interrupts in mstatus
    }
}
