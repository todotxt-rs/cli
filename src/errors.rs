pub type Result<T = ()> = anyhow::Result<T>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Env(#[from] envir::Error),
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Parser(#[from] todo_txt::Error),
}
