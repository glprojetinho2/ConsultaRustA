use core::error;

// TODO: usar wrap em vez de boxing
pub type Result<T> = std::result::Result<T, Box<dyn error::Error>>;
