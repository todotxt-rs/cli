#[derive(envir::Deserialize)]
#[envir(prefix = "TODOTXT_")]
pub struct Config {
    #[envir(nested)]
    inner: todo_txt::Config,
    #[envir(default = "true")]
    pub reldate: bool,
    #[envir(default = "14")]
    pub reldate_dayrange: usize,
}

impl Config {
    pub fn from_env() -> Self {
        todo_txt::Config::load_env();

        envir::from_env().unwrap()
    }
}

impl std::ops::Deref for Config {
    type Target = todo_txt::Config;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<&crate::Opt> for Config {
    fn from(value: &crate::Opt) -> Self {
        let mut config = Self::from_env();

        config.inner.auto_archive |= !value.dont_auto_archive;
        config.inner.date_on_add |= value.append_current_date;
        config.inner.disable_filter |= value.disable_final_filter;
        config.inner.force |= value.force;
        config.inner.plain |= value.plain_text;
        config.inner.preserve_line_numbers |= !value.dont_preserve_line_numbers;
        config.inner.verbose |= value.verbose;

        config
    }
}
