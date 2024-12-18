use anyhow::Context;

pub struct List {
    filename: String,
    inner: todo_txt::task::List<crate::Task>,
}

impl List {
    pub fn from(filename: &str) -> crate::Result<crate::List> {
        if !std::path::Path::new(filename).exists() {
            std::fs::File::create(filename)
                .with_context(|| format!("Failed to create '{filename}' file"))?;
        }

        let contents = std::fs::read_to_string(filename)
            .with_context(|| format!("Failed to read '{filename}' file"))?;


        let list = Self {
            filename: filename.to_string(),
            inner: todo_txt::task::List::from(&contents),
        };

        Ok(list)
    }

    pub fn save(&self) -> crate::Result {
        std::fs::write(&self.filename, self.inner.to_string().as_bytes())
            .with_context(|| format!("Failed to save in '{}' file", self.filename))
    }
}

impl std::ops::Deref for List {
    type Target = todo_txt::task::List<crate::Task>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::DerefMut for List {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
