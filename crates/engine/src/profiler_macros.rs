#[cfg(feature="profiler")]
macro_rules! profile_function {
    ($($arg:expr)?) => {
        puffin::profile_function!($($arg)?);
    };
}
#[cfg(feature="profiler")]
macro_rules! profile_scope {
    ($id:expr $(,$arg:expr)?) => {
        puffin::profile_scope!($id $(,$arg)?);
    };
}

#[cfg(not(feature="profiler"))]
macro_rules! profile_function {
    ($($arg:expr)?) => {
        
    };
}
#[cfg(not(feature="profiler"))]
macro_rules! profile_scope {
    ($id:expr $(,$arg:expr)?) => {
        
    };
}
