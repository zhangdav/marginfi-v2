#[macro_export]
macro_rules! check {
    ($cond:expr, $err:expr) => {
        if !($cond) {
            let error_code: $crate::errors::MarginfiError = $err;
            #[cfg(not(feature = "test_bpf"))]
            anchor_lang::prelude::msg!(
                "Error \"{}\" thrown at {}:{}",
                error_code,
                file!(),
                line!()
            );
            return Err(error_code.into());
        }
    }
}

// Check the memory size of the structure
#[macro_export]
macro_rules! assert_struct_size {
    ($struct: ty, $size: expr) => {
        static_assertions::const_assert_eq!(std::mem::size_of::<$struct>(), $size);
    };
}

// Check the memory alignment of the structure
#[macro_export]
macro_rules! assert_struct_align {
    ($struct: ty, $align: expr) => {
        static_assertions::const_assert_eq!(std::mem::align_of::<$struct>(), $align);
    };
}