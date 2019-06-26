#![macro_use]
#![doc(hidden)]
macro_rules! create_error {
    ( $name: ident, $message: expr) => {
        #[derive(Debug)]
        #[doc(hidden)]
        pub struct $name;

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, $message)
            }
        }

        impl std::error::Error for $name {
            fn description(&self) -> &str {
                $message
            }
        }
    };
}
