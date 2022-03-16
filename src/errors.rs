pub type Result<T = ()> = anyhow::Result<T>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("The {0} environment variable is not set.")]
    Env(String),
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    List(String),
    #[error("{0}")]
    Parser(#[from] todo_txt::Error),
}
