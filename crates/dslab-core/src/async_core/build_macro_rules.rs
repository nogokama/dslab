#[macro_export]
macro_rules! async_core {
    ($($item:item)*) => {
        $(#[cfg(feature = "async_core")]
        $item)*
    }
}

#[macro_export]
macro_rules! async_only_core {
    ($($item:item)*) => {
        $(#[cfg(all(feature = "async_core", not(feature = "async_details_core")))]
        $item)*
    }
}

#[macro_export]
macro_rules! async_details_core {
    ($($item:item)*) => {
        $(#[cfg(feature = "async_details_core")]
        $item)*
    }
}

#[macro_export]
macro_rules! async_disabled {
    ($($item:item)*) => {
        $(#[cfg(not(feature = "async_core"))]
        $item)*
    }
}
