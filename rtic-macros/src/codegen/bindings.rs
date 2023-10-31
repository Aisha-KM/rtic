#[cfg(not(any(
    feature = "cortex-m-source-masking",
    feature = "cortex-m-basepri",
    feature = "test-template",
    feature = "riscv-esp32c3",
    feature = "riscv-cva6"
)))]
compile_error!("No backend selected");

#[cfg(any(feature = "cortex-m-source-masking", feature = "cortex-m-basepri"))]
pub use cortex::*;

#[cfg(feature = "test-template")]
pub use template::*;

#[cfg(any(feature = "cortex-m-source-masking", feature = "cortex-m-basepri"))]
mod cortex;

#[cfg(feature = "test-template")]
mod template;

#[cfg(feature = "riscv-esp32c3")]
pub use esp32c3::*;

#[cfg(feature = "riscv-esp32c3")]
mod esp32c3;

#[cfg(feature = "riscv-cva6")]
pub use cva6::*;

#[cfg(feature = "riscv-cva6")]
mod cva6;
