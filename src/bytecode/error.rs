use thiserror::Error;

pub type Result<T, E = LoxError> = std::result::Result<T, E>;

#[derive(Error, Debug)]
pub enum LoxError {}
