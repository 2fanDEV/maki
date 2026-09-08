/// Returns an error when a condition is false, forwarding to `anyhow::ensure!`.
#[macro_export]
macro_rules! invariant {
    ($($args:tt)*) => {
        ::anyhow::ensure!($($args)*);
    };
}
