pub(crate) fn todo_file(current: &std::ffi::OsStr) -> Vec<clap_complete::CompletionCandidate> {
    let mut completions = Vec::new();
    let current = current.to_str().unwrap_or_default();

    let config = crate::Config::from_env();
    let Ok(dir) = std::fs::read_dir(&config.todo_dir) else {
        return completions;
    };

    let txt_ext = std::ffi::OsStr::new("txt");

    for entry in dir {
        let Ok(entry) = entry else {
            continue;
        };

        let file_name = entry.file_name().into_string().unwrap_or_default();

        if !file_name.starts_with(current) {
            continue;
        }

        if entry.path().extension() != Some(txt_ext) {
            continue;
        }

        completions.push(clap_complete::CompletionCandidate::new(file_name));
    }

    completions
}

pub(crate) fn all_item(current: &std::ffi::OsStr) -> Vec<clap_complete::CompletionCandidate> {
    item(current, |_, _| true)
}

pub(crate) fn note(current: &std::ffi::OsStr) -> Vec<clap_complete::CompletionCandidate> {
    item(current, |_, x| x.has_note())
}

fn item<F>(current: &std::ffi::OsStr, filter: F) -> Vec<clap_complete::CompletionCandidate>
where
    F: Fn(usize, &crate::Task) -> bool,
{
    let config = crate::Config::from_env();
    let todo = crate::List::from(&config.todo_file).unwrap();
    let current = current.to_str().unwrap_or_default();

    todo.iter()
        .enumerate()
        .map(|(k, v)| (k + 1, v))
        .filter(|(k, v)| filter(*k, v))
        .map(|(k, _)| k.to_string())
        .filter(|k| k.starts_with(current))
        .map(clap_complete::CompletionCandidate::new)
        .collect()
}

pub(crate) fn filter(current: &std::ffi::OsStr) -> Vec<clap_complete::CompletionCandidate> {
    let current = current.to_str().unwrap_or_default();

    let Some(last_item) = current.split(' ').next_back() else {
        return Vec::new();
    };

    if let Some(project) = last_item.strip_prefix('+') {
        tag(project, todo_txt::task::List::projects)
    } else if let Some(context) = last_item.strip_prefix('@') {
        tag(context, todo_txt::task::List::contexts)
    } else {
        Vec::new()
    }
}

pub(crate) fn context(current: &std::ffi::OsStr) -> Vec<clap_complete::CompletionCandidate> {
    tag(
        current.to_str().unwrap_or_default(),
        todo_txt::task::List::contexts,
    )
}

pub(crate) fn project(current: &std::ffi::OsStr) -> Vec<clap_complete::CompletionCandidate> {
    tag(
        current.to_str().unwrap_or_default(),
        todo_txt::task::List::projects,
    )
}

fn tag<F: Fn(&todo_txt::task::List<crate::Task>) -> Vec<String>>(
    current: &str,
    f: F,
) -> Vec<clap_complete::CompletionCandidate> {
    let config = crate::Config::from_env();
    let todo = crate::List::from(&config.todo_file).unwrap();

    f(&todo)
        .iter()
        .filter_map(|x| {
            if x.starts_with(current) {
                Some(clap_complete::CompletionCandidate::new(x))
            } else {
                None
            }
        })
        .collect()
}

pub(crate) fn pri(current: &std::ffi::OsStr) -> Vec<clap_complete::CompletionCandidate> {
    if !current.is_empty() {
        return Vec::new();
    }

    ('A'..='Z')
        .map(|x| clap_complete::CompletionCandidate::new(x.to_string()))
        .collect()
}
