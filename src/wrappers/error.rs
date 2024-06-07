#[derive(Debug, thiserror::Error)]
pub enum WrapperError {
    #[error("{} wrapper is not set", .0)]
    NoWrapper(&'static str),

    #[error("{} is forbidden when {} wrapper is set", .action, .wrapper)]
    Forbidden { action: String, wrapper: String },
}
