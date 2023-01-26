use anyhow::Context;

#[derive(Clone, Debug, Default)]
pub struct List {
    filename: String,
    tasks: Vec<crate::Task>,
}

impl List {
    pub fn from(filename: &str) -> crate::Result<Self> {
        use std::io::Read;

        if !std::path::Path::new(filename).exists() {
            std::fs::File::create(filename)
                .with_context(|| format!("Failed to create '{filename}' file"))?;
        }

        let mut file = std::fs::File::open(filename)
            .with_context(|| format!("Failed to open '{filename}' file"))?;

        let mut content = String::new();
        file.read_to_string(&mut content)
            .with_context(|| format!("Failed to read '{filename}' file"))?;

        let list = Self {
            filename: filename.to_string(),
            tasks: Self::load(&content)
                .with_context(|| format!("Load task list from {filename}"))?,
        };

        Ok(list)
    }

    fn load(content: &str) -> crate::Result<Vec<crate::Task>> {
        let mut tasks = Vec::new();

        for line in content.lines() {
            if line.is_empty() {
                continue;
            }

            match line.parse() {
                Ok(task) => tasks.push(task),
                Err(_) => {
                    return Err(anyhow::anyhow!(crate::Error::List(format!(
                        "Invalid tasks: '{line}'"
                    ))));
                }
            };
        }

        Ok(tasks)
    }

    pub fn save(&mut self) -> crate::Result {
        std::fs::write(&self.filename, self.to_string().as_bytes())
            .with_context(|| format!("Failed to save in '{}' file", self.filename))?;

        Ok(())
    }

    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    pub fn get(&self, index: &usize) -> &crate::Task {
        &self.tasks[*index - 1]
    }

    pub fn get_mut(&mut self, index: &usize) -> &mut crate::Task {
        &mut self.tasks[*index - 1]
    }

    pub fn remove(&mut self, index: usize) -> crate::Task {
        self.tasks.remove(index - 1)
    }

    pub fn push(&mut self, task: crate::Task) {
        self.tasks.push(task);
    }

    pub fn iter(&self) -> std::slice::Iter<'_, crate::Task> {
        self.tasks.iter()
    }

    pub fn dedup(&mut self) {
        self.tasks.sort();
        self.tasks.dedup();
    }
}

impl ToString for List {
    fn to_string(&self) -> String {
        use std::fmt::Write as _;

        let mut s = String::new();

        for task in &self.tasks {
            writeln!(s, "{task}").ok();
        }

        s
    }
}
