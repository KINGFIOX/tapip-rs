macro_rules! net_log {
    (trace, $($arg:expr),*) => { println!($($arg),*) };
    (debug, $($arg:expr),*) => { println!($($arg),*) };
}

#[allow(unused_macros)]
macro_rules! net_trace {
    ($($arg:expr),*) => (net_log!(trace, $($arg),*));
}

macro_rules! net_debug {
    ($($arg:expr),*) => (net_log!(debug, $($arg),*));
}
