use std::fmt::Write as _;

struct Summary {
    file: String,
    total: usize,
    show: usize,
}

impl std::fmt::Display for Summary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {} of {} tasks show",
            prefix(&self.file),
            self.show,
            self.total
        )
    }
}

pub(crate) fn add(config: &crate::Config, add: &crate::opts::Add) -> crate::Result {
    add_tasks(config, &config.todo_file, add)
}

pub(crate) fn addm(config: &crate::Config, add: &crate::opts::Add) -> crate::Result {
    add_tasks(config, &config.todo_file, add)
}

pub(crate) fn addto(
    config: &crate::Config,
    crate::opts::AddTo { dest, add }: &crate::opts::AddTo,
) -> crate::Result {
    let file = format!("{}/{dest}", config.todo_dir);

    add_tasks(config, &file, add)
}

fn add_tasks(
    config: &crate::Config,
    dest: &str,
    crate::opts::Add { task }: &crate::opts::Add,
) -> crate::Result {
    let mut summary = String::new();
    let mut list = crate::List::from(dest)?;

    let tasks = if task.is_empty() {
        ask(config, "Add:")?
    } else {
        task.join(" ")
    };

    for task in tasks.split('\n') {
        let mut todo: crate::Task = task.parse()?;

        if config.date_on_add && todo.create_date.is_none() {
            let today = todo_txt::date::today();
            todo.create_date = Some(today);
        }

        if let Some(pri) = config.priority_on_add {
            if todo.priority.is_lowest() {
                todo.priority = pri.try_into().unwrap_or_default();
            }
        }

        list.push(todo);

        writeln!(summary, "{} {task}", list.len())?;
    }

    list.save()?;

    print!("{summary}");

    Ok(())
}

pub(crate) fn append(
    config: &crate::Config,
    crate::opts::Append { item, add }: &crate::opts::Append,
) -> crate::Result {
    let mut list = crate::List::from(&config.todo_file)?;

    let text = if add.task.is_empty() {
        ask(config, "Append:")?
    } else {
        add.task.join(" ")
    };

    write!(list.get_mut(item).subject, " {text}")?;

    list.save()?;

    if config.verbose > 0 {
        println!("{item} {}", list.get(item));
    }

    Ok(())
}

pub(crate) fn archive(config: &crate::Config) -> crate::Result {
    let mut todo = crate::List::from(&config.todo_file)?;
    let mut done = crate::List::from(&config.done_file)?;

    let mut i = 0;

    // @FIXME feature(drain_filter)
    while i < todo.len() {
        let index = i + 1;

        if todo.get(&index).finished {
            let mut task = todo.remove(index);
            archive_note(config, &mut task)?;
            done.push(task);
        } else {
            i += 1;
        }
    }

    todo.save()?;
    done.save()?;

    if config.verbose > 0 {
        println!("TODO: {} archived", config.todo_file);
    }

    Ok(())
}

#[cfg(not(feature = "extended"))]
fn archive_note(_: &crate::Config, _: &mut crate::Task) -> crate::Result {
    Ok(())
}

#[cfg(feature = "extended")]
fn archive_note(config: &crate::Config, task: &mut crate::Task) -> crate::Result {
    if let Some(note) = task.note.content() {
        use std::io::Write;

        let mut archive = std::fs::File::options()
            .append(true)
            .open(&config.note_archive)?;

        archive.write_all(format!("{note}\n").as_bytes())?;
    }

    task.note.delete()?;

    Ok(())
}

pub(crate) fn deduplicate(config: &crate::Config) -> crate::Result {
    let mut todo = crate::List::from(&config.todo_file)?;
    let original_task_num = todo.len();

    todo.dedup();
    todo.save()?;

    let deduplicate_num = original_task_num - todo.len();

    if deduplicate_num == 0 {
        println!("TODO: No duplicate tasks found");
    } else {
        println!("TODO: {deduplicate_num} duplicate task(s) removed");
    }

    Ok(())
}

pub(crate) fn del(
    config: &crate::Config,
    crate::opts::Del { item, filter }: &crate::opts::Del,
) -> crate::Result {
    if filter.term.is_none() {
        if !confirm(config, &format!("Delete {item}"))? {
            return Ok(());
        }

        let mut todo = crate::List::from(&config.todo_file)?;

        let task = todo.remove(*item);

        todo.save()?;

        if config.verbose > 0 {
            println!("{item} {task}");
            println!("TODO: {item} deleted.");
        }
    } else {
        todo!()
    }

    Ok(())
}

pub(crate) fn delpri(
    config: &crate::Config,
    crate::opts::Item { item }: &crate::opts::Item,
) -> crate::Result {
    let mut todo = crate::List::from(&config.todo_file)?;

    todo.get_mut(item).priority = todo_txt::Priority::lowest();
    todo.save()?;

    if config.verbose > 0 {
        println!("{item} {}", todo.get(item));
        println!("TODO: {item} deprioritized.");
    }

    Ok(())
}

pub(crate) fn done(
    config: &crate::Config,
    crate::opts::Item { item }: &crate::opts::Item,
) -> crate::Result {
    let mut todo = crate::List::from(&config.todo_file)?;

    let task = if config.auto_archive {
        let mut task = todo.remove(*item);
        task.finished = true;

        let mut done = crate::List::from(&config.done_file)?;
        done.push(task.clone());
        done.save()?;

        task
    } else {
        todo.get_mut(item).finished = true;

        todo.get(item).clone()
    };

    recurrence(config, &mut todo, &task);
    todo.save()?;

    if config.verbose > 0 {
        println!("{item} {task}");
        println!("TODO: {item} marked as done.");
    }

    Ok(())
}

#[cfg(not(feature = "extended"))]
fn recurrence(_: &crate::Config, _: &mut crate::List, _: &crate::Task) {}

#[cfg(feature = "extended")]
fn recurrence(config: &crate::Config, todo: &mut crate::List, task: &crate::Task) {
    if let Some(ref recurrence) = task.recurrence {
        let due = if recurrence.strict && task.due_date.is_some() {
            task.due_date.unwrap()
        } else {
            todo_txt::date::today()
        };

        let mut new = task.clone();
        new.uncomplete();
        if config.date_on_add {
            new.create_date = Some(todo_txt::date::today());
        }
        new.due_date = Some(recurrence.clone() + due);

        if let Some(threshold_date) = task.threshold_date {
            new.threshold_date = Some(recurrence.clone() + threshold_date);
        }

        todo.push(new);
    }
}

pub(crate) fn flag(config: &crate::Config, item: usize) -> crate::Result {
    let mut list = crate::List::from(&config.todo_file)?;
    let task = list.get_mut(&item);
    task.flagged = true;

    list.save()
}

pub(crate) fn listflag(config: &crate::Config) -> crate::Result {
    let summary = print_list(config, true, &config.todo_file, |(_, x)| x.flagged)?;

    print_summary(&[summary]);

    Ok(())
}

pub(crate) fn list(config: &crate::Config, filter: &crate::opts::Filter) -> crate::Result {
    let now = todo_txt::date::today();

    let summary = print_list(config, true, &config.todo_file, |(_, x)| {
        !x.finished
            && filter_hidden(x)
            && filter_term(&x.subject, filter)
            && now >= x.threshold_date.unwrap_or(now)
    })?;

    print_summary(&[summary]);

    Ok(())
}

pub(crate) fn listall(config: &crate::Config, filter: &crate::opts::Filter) -> crate::Result {
    let summary = vec![
        print_list(config, true, &config.todo_file, |(_, x)| {
            filter_term(&x.subject, filter)
        })?,
        print_list(config, false, &config.done_file, |(_, x)| {
            filter_term(&x.subject, filter)
        })?,
    ];

    print_summary(&summary);

    Ok(())
}

pub(crate) fn listaddons(config: &crate::Config) -> crate::Result {
    let mut entries = std::fs::read_dir(&config.action_dir)?
        .map(|x| x.map(|e| e.file_name().to_string_lossy().to_string()))
        .collect::<Result<Vec<_>, std::io::Error>>()?;

    entries.sort();

    println!("{}", entries.join("\n"));

    Ok(())
}

pub(crate) fn listfile(
    config: &crate::Config,
    crate::opts::ListFile { src, filter }: &crate::opts::ListFile,
) -> crate::Result {
    let file = format!("{}/{}", config.todo_dir, src);
    let summary = print_list(config, true, &file, |(_, x)| {
        filter_term(&x.subject, filter)
    })?;

    print_summary(&[summary]);

    Ok(())
}

fn print_list<P>(
    config: &crate::Config,
    with_id: bool,
    file: &str,
    predicate: P,
) -> crate::Result<Summary>
where
    P: FnMut(&(usize, &crate::Task)) -> bool,
{
    let list = crate::List::from(file)?;
    let total = list.len();

    let width = total.ilog10() as usize + 1;

    let tasks = list
        .iter()
        .enumerate()
        .map(|(id, task)| if with_id { (id + 1, task) } else { (0, task) })
        .filter(predicate)
        .map(|task| print(config, width, task))
        .collect::<Vec<_>>()
        .join("\n");

    let filtered_tasks = exec(&config.final_filter, tasks)?;
    let sorted_tasks = exec(&config.sort_command, filtered_tasks)?;

    let show = sorted_tasks.lines().count();

    print!("{sorted_tasks}");
    if config.verbose > 1 {
        println!("TODO DEBUG: Filter Command was: {}", config.final_filter);
    }

    Ok(Summary {
        file: file.to_string(),
        total,
        show,
    })
}

fn print_summary(summary: &[Summary]) {
    let mut show = 0;
    let mut total = 0;

    println!("--");

    for s in summary {
        println!("{s}");
        show += s.show;
        total += s.total;
    }

    if summary.len() > 1 {
        println!("total: {show} of {total} tasks show");
    }
}

fn exec(command: &str, input: String) -> crate::Result<String> {
    use std::io::Write;

    let mut command = std::process::Command::new("bash")
        .arg("-c")
        .arg(command)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()?;

    let mut stdin = command.stdin.take().unwrap();

    std::thread::spawn(move || {
        stdin.write_all(input.as_bytes()).unwrap();
    });

    let output = command.wait_with_output()?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn prefix(filename: &str) -> String {
    let path = std::path::PathBuf::from(filename);

    path.file_stem().unwrap().to_string_lossy().to_uppercase()
}

#[cfg(not(feature = "extended"))]
fn filter_hidden(_: &crate::Task) -> bool {
    true
}

#[cfg(feature = "extended")]
fn filter_hidden(task: &crate::Task) -> bool {
    !task.hidden
}

fn filter_term(s: &str, crate::opts::Filter { term }: &crate::opts::Filter) -> bool {
    let Some(terms) = term else {
        return true;
    };

    let mut accept = false;

    for term in terms.split('|') {
        if term.starts_with('-') {
            let filter = term.trim_start_matches('-');

            if !s.to_lowercase().contains(&filter.to_lowercase()) {
                accept |= true;
            }
        } else if s.to_lowercase().contains(&term.to_lowercase()) {
            accept |= true;
        }
    }

    accept
}

fn print(config: &crate::Config, width: usize, (id, task): (usize, &crate::Task)) -> String {
    let mut output = format!("{id:0width$} ");

    if task.finished {
        output.push_str("x ");
    }

    if let Some(finish_date) = task.finish_date {
        write!(
            output,
            "{} ",
            config.colors.date.colorize(&finish_date.to_string())
        )
        .ok();
    }

    if let Some(create_date) = task.create_date {
        write!(
            output,
            "{} ",
            config.colors.date.colorize(&create_date.to_string())
        )
        .ok();
    }

    if !task.priority.is_lowest() {
        write!(output, "({}) ", task.priority).ok();
    }

    #[cfg(feature = "extended")]
    if task.flagged {
        output.push_str(" ");
    }

    #[cfg(feature = "extended")]
    if task.has_note() {
        output.push_str(" ");
    }

    let regex = regex::Regex::new(r#"(?P<number>[0-9]+)"#).unwrap();
    let subject = regex.replace_all(&task.subject, |caps: &regex::Captures| {
        config.colors.number.colorize(&caps["number"])
    });

    let regex = regex::Regex::new(r"(?P<label>(?P<type>\+|@)[^ ]+)").unwrap();
    let subject = regex.replace_all(&subject, |caps: &regex::Captures| {
        let color = match &caps["type"] {
            "+" => config.colors.project.clone(),
            "@" => config.colors.context.clone(),
            _ => crate::Color::None,
        };

        color.colorize(&caps["label"])
    });

    let regex = regex::Regex::new(
        r#"(?P<date>(19|20)[0-9][0-9]-(0[1-9]|1[012])-(0[1-9]|[12][0-9]|3[01]))"#,
    )
    .unwrap();
    let subject = regex.replace_all(&subject, |caps: &regex::Captures| {
        config.colors.date.colorize(&caps["date"])
    });

    output.push_str(&subject);

    let color = &config.colors.meta;

    if let Some(due_date) = task.due_date {
        output.push_str(&color.colorize(&format!(" due:{}", due_date.format("%Y-%m-%d"))));
    }

    if let Some(threshold_date) = task.threshold_date {
        output.push_str(&color.colorize(&format!(" t:{}", threshold_date.format("%Y-%m-%d"))));
    }

    #[cfg(feature = "extended")]
    if let Some(recurrence) = &task.recurrence {
        output.push_str(&color.colorize(&format!(" rec:{recurrence}")));
    }

    for (key, value) in &task.tags {
        output.push_str(&color.colorize(&format!(" {key}:{value}")));
    }

    if task.finished {
        config.colors.done.colorize(&output)
    } else if task.priority.is_lowest() {
        output
    } else {
        let color = config.colors.for_pri(&task.priority);

        color.colorize(&output)
    }
}

macro_rules! list_tag {
    ($ty:ident, $config:ident, $filter:ident) => {{
        let todo = crate::List::from(&$config.todo_file)?;

        let mut tags = todo
            .iter()
            .map(|x| x.$ty().clone())
            .flatten()
            .collect::<Vec<_>>();

        tags.dedup();

        for tag in tags {
            if filter_term(&tag, $filter) {
                println!("{tag}");
            }
        }

        Ok(())
    }};
}

pub(crate) fn listcon(config: &crate::Config, filter: &crate::opts::Filter) -> crate::Result {
    list_tag!(contexts, config, filter)
}

pub(crate) fn listpri(
    config: &crate::Config,
    crate::opts::ListPri { priority, filter }: &crate::opts::ListPri,
) -> crate::Result {
    let summary = print_list(config, true, &config.todo_file, |(_, x)| {
        if !filter_term(&x.subject, filter) {
            return false;
        }

        if let Some(p) = priority {
            x.priority == *p
        } else {
            !x.priority.is_lowest()
        }
    })?;

    print_summary(&[summary]);

    Ok(())
}

pub(crate) fn listproj(config: &crate::Config, filter: &crate::opts::Filter) -> crate::Result {
    list_tag!(projects, config, filter)
}

pub(crate) fn r#move(
    config: &crate::Config,
    crate::opts::Move { item, dest, src }: &crate::opts::Move,
) -> crate::Result {
    if !confirm(config, &format!("Move {item} form {src} to {dest}"))? {
        return Ok(());
    }

    let src_file = format!("{}/{src}", config.todo_dir);
    let mut src_list = crate::List::from(&src_file)?;
    let dest_file = format!("{}/{dest}", config.todo_dir);
    let mut dest_list = crate::List::from(&dest_file)?;

    let task = src_list.remove(*item);
    dest_list.push(task.clone());

    src_list.save()?;
    dest_list.save()?;

    if config.verbose > 0 {
        println!("{item} {task}");
        println!("TODO: {item} item moved from '{src_file}' to '{dest_file}'");
    }

    Ok(())
}

#[cfg(feature = "extended")]
pub(crate) fn note(config: &crate::Config, subcommand: &crate::opts::Note) -> crate::Result {
    match subcommand {
        crate::opts::Note::Add(item) => note_add(config, item),
        crate::opts::Note::Archive => note_archive(config),
        crate::opts::Note::Edit(item) => note_edit(config, item),
        crate::opts::Note::Show(args) => note_show(config, args),
    }
}

#[cfg(feature = "extended")]
pub(crate) fn note_add(
    config: &crate::Config,
    crate::opts::Item { item }: &crate::opts::Item,
) -> crate::Result {
    let mut list = crate::List::from(&config.todo_file)?;

    let todo = list.get_mut(item);
    todo.note = todo_txt::task::Note::Short(String::new());
    todo.note.write()?;

    list.save()?;

    println!("TODO: Note added to task {item}");

    if !config.force && confirm(config, "Edit note?")? {
        note_edit(config, &crate::opts::Item { item: *item })?;
    }

    Ok(())
}

#[cfg(feature = "extended")]
pub(crate) fn note_edit(
    config: &crate::Config,
    crate::opts::Item { item }: &crate::opts::Item,
) -> crate::Result {
    let editor = envir::get("EDITOR")?;

    let list = crate::List::from(&config.todo_file)?;

    let todo_txt::task::Note::Long { filename, .. } = &list.get(item).note else {
        println!("TODO: Task {item} has no note.");
        return Ok(());
    };

    std::process::Command::new(editor)
        .arg(format!("{}/{filename}", config.notes_dir))
        .status()?;

    Ok(())
}

#[cfg(feature = "extended")]
pub(crate) fn note_show(
    config: &crate::Config,
    crate::opts::Item { item }: &crate::opts::Item,
) -> crate::Result {
    let list = crate::List::from(&config.todo_file)?;

    if let Some(note) = list.get(item).note.content() {
        let note = exec(&config.note_filter, note)?;
        print!("{note}");
    } else {
        println!("TODO: Task {item} has no note.");
    }

    Ok(())
}

#[cfg(feature = "extended")]
pub(crate) fn note_archive(config: &crate::Config) -> crate::Result {
    println!(
        "{}",
        String::from_utf8_lossy(&std::fs::read(&config.note_archive)?)
    );

    Ok(())
}

pub(crate) fn prepend(
    config: &crate::Config,
    crate::opts::Append { item, add }: &crate::opts::Append,
) -> crate::Result {
    let mut list = crate::List::from(&config.todo_file)?;

    let mut text = if add.task.is_empty() {
        ask(config, "Prepend:")?
    } else {
        add.task.join(" ")
    };

    if !text.ends_with(' ') {
        text.push(' ');
    }

    list.get_mut(item).subject.insert_str(0, &text);

    list.save()?;

    if config.verbose > 0 {
        println!("{item} {}", list.get(item));
    }

    Ok(())
}

pub(crate) fn pri(
    config: &crate::Config,
    crate::opts::Pri { item, priority }: &crate::opts::Pri,
) -> crate::Result {
    let mut list = crate::List::from(&config.todo_file)?;

    let task = list.get_mut(item);
    let oldpri = task.priority.clone();
    task.priority = (*priority).try_into()?;

    list.save()?;

    if config.verbose > 0 {
        let task = list.get(item);

        println!("{item} {task}");

        if oldpri.is_lowest() {
            println!("TODO: {item} prioritized ({})", task.priority);
        } else {
            println!(
                "TODO: {item} re-prioritized from ({oldpri}) to ({}).",
                task.priority
            );
        }
    }

    Ok(())
}

pub(crate) fn replace(
    config: &crate::Config,
    crate::opts::Replace { item, text }: &crate::opts::Replace,
) -> crate::Result {
    let mut list = crate::List::from(&config.todo_file)?;

    let text = match text {
        Some(text) => text.clone(),
        None => ask(config, "Replace:")?,
    };

    let old_task = list.get(item).clone();
    (*list.get_mut(item)) = text.parse()?;

    list.save()?;

    if config.verbose > 0 {
        let new_task = list.get(item);

        println!("{item} {old_task}");
        println!("TODO: Replaced task with:");
        println!("{item} {new_task}");
    }

    Ok(())
}

pub(crate) fn report(config: &crate::Config) -> crate::Result {
    use std::io::Write;

    let todo = crate::List::from(&config.todo_file)?;
    let done = crate::List::from(&config.done_file)?;

    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(&config.report_file)?;

    let now = chrono::offset::Local::now();

    file.write_all(format!("{} {} {}\n", now.format("%FT%X"), todo.len(), done.len()).as_bytes())?;

    if config.verbose > 0 {
        println!("TODO: Report file updated.");
    }

    Ok(())
}

pub(crate) fn external(config: &crate::Config, args: &[String]) -> crate::Result {
    use anyhow::Context;

    let command = format!("{}/{}", config.action_dir, args[0]);

    std::process::Command::new(&command)
        .args(args)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .output()
        .with_context(|| format!("Unable to execute {command}"))?;

    Ok(())
}

fn confirm(config: &crate::Config, question: &str) -> crate::Result<bool> {
    ask(config, &format!("{question}: (y/n)")).map(|x| x == "y\n" || (config.force && x.is_empty()))
}

fn ask(config: &crate::Config, question: &str) -> crate::Result<String> {
    use std::io::Write;

    if config.force {
        return Ok(String::new());
    }

    print!("{question} ");
    std::io::stdout().flush()?;

    let mut answer = String::new();
    let stdin = std::io::stdin();
    stdin.read_line(&mut answer)?;

    Ok(answer)
}
