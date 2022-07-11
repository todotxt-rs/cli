use std::collections::HashMap;

#[derive(Debug)]
struct Result {
    todo_dir: std::path::PathBuf,
    stdout: String,
    todo: String,
    done: String,
    report: String,
}

#[test]
fn action() {
    use std::os::unix::prelude::PermissionsExt;

    let todo_dir = create_dir();
    let command = todo_dir.join("ping");

    std::fs::write(&command, "#!/bin/bash\necho 'pong'").unwrap();
    std::fs::set_permissions(&command, std::fs::Permissions::from_mode(0o755)).unwrap();

    let result = reexec(todo_dir, "ping", &[]);
    assert_eq!(result.stdout, "pong\n");
}

#[test]
fn add() {
    let result = exec("add", &["new", "task"]);
    assert_eq!(result.todo, format!("new task\n"));
    assert_eq!(result.stdout, format!("1 new task\n"));
}

#[test]
fn addm() {
    let task = "new task 1\nnew task 2";

    let result = exec("addm", &[task]);
    assert_eq!(result.todo, format!("{task}\n"));
}

#[test]
fn addto() {
    let task = "new task";

    let result = exec("addto", &["done.txt", task]);
    assert_eq!(result.done, format!("{task}\n"));
}

#[test]
fn append() {
    let task = "new task";
    let Result { todo_dir, .. } = exec("add", &[task]);

    let result = reexec(todo_dir, "append", &["1", "with extra text"]);
    assert_eq!(result.todo, "new task with extra text\n");
    assert_eq!(result.stdout, "1 new task with extra text\n");
}

#[test]
fn archive() {
    let task = "new task\nx done task";
    let Result { todo_dir, .. } = exec("addm", &[task]);

    let result = reexec(todo_dir, "archive", &[]);
    assert_eq!(result.todo, "new task\n");
    assert_eq!(result.done, "x done task\n");
    assert_eq!(
        result.stdout,
        format!("TODO: {}/todo.txt archived\n", result.todo_dir.display())
    );
}

#[test]
fn deduplicate() {
    let task = "new task 1\nnew task 2\nnew task 1";
    let Result { todo_dir, .. } = exec("addm", &[task]);

    let result = reexec(todo_dir, "deduplicate", &[]);
    assert_eq!(result.todo, "new task 1\nnew task 2\n");
    assert_eq!(result.stdout, "TODO: 1 duplicate task(s) removed\n");
}

#[test]
fn del() {
    let task = "new task 1\nnew task 2";
    let Result { todo_dir, .. } = exec("addm", &[task]);

    let result = reexec(todo_dir, "del", &["1"]);
    assert_eq!(result.todo, "new task 2\n");
    assert_eq!(result.stdout, "1 new task 1\nTODO: 1 deleted.\n");
}

#[test]
fn del_term() {}

#[test]
fn delpri() {
    let task = "(A) new task 1\n(B) new task 2";
    let Result { todo_dir, .. } = exec("addm", &[task]);

    let result = reexec(todo_dir, "delpri", &["2"]);
    assert_eq!(result.todo, "(A) new task 1\nnew task 2\n");
    assert_eq!(result.stdout, "2 new task 2\nTODO: 2 deprioritized.\n");
}

#[test]
fn done() {
    let task = "new task 1\nnew task 2";
    let Result { todo_dir, .. } = exec("addm", &[task]);

    let result = reexec(todo_dir, "done", &["2"]);
    assert_eq!(result.todo, "new task 1\n");
    assert_eq!(result.done, "x new task 2\n");
    assert_eq!(result.stdout, "2 x new task 2\nTODO: 2 marked as done.\n");
}

#[test]
fn list() {
    let task = "new task 1\nnew task 2";
    let Result { todo_dir, .. } = exec("addm", &[task]);

    let result = reexec(todo_dir, "list", &[]);
    assert_eq!(
        result.stdout,
        "1 new task 1\n2 new task 2\n--\nTODO: 2 of 2 tasks show\n"
    );
}

#[test]
#[cfg(feature = "extended")]
fn hidden() {
    let task = "new task 1\nnew task 2\nhidden task h:1";
    let Result { todo_dir, .. } = exec("addm", &[task]);

    let result = reexec(todo_dir, "list", &[]);
    assert_eq!(
        result.stdout,
        "1 new task 1\n2 new task 2\n--\nTODO: 2 of 3 tasks show\n"
    );
}

#[test]
#[cfg(feature = "extended")]
fn flag() {
    let task = "new task 1";
    let Result { todo_dir, .. } = exec("add", &[task]);

    let result = reexec(todo_dir, "flag", &[]);
    assert_eq!(
        result.stdout,
        "--\nTODO: 0 of 1 tasks show\n"
    );

    let result = reexec(result.todo_dir, "flag", &["1"]);

    let result = reexec(result.todo_dir, "flag", &[]);
    assert_eq!(
        result.stdout,
        "1 🚩 new task 1\n--\nTODO: 1 of 1 tasks show\n"
    );
}

#[test]
fn filter() {
    let task = "new task 1\nnew task 2\nnew task 3";
    let Result { todo_dir, .. } = exec("addm", &[task]);

    let result = reexec(todo_dir, "list", &["2"]);
    assert_eq!(result.stdout, "2 new task 2\n--\nTODO: 1 of 3 tasks show\n");

    let result = reexec(result.todo_dir, "list", &["--", "-2"]);
    assert_eq!(
        result.stdout,
        "1 new task 1\n3 new task 3\n--\nTODO: 2 of 3 tasks show\n"
    );

    let result = reexec(result.todo_dir, "list", &["2|3"]);
    assert_eq!(
        result.stdout,
        "2 new task 2\n3 new task 3\n--\nTODO: 2 of 3 tasks show\n"
    );
}

#[test]
fn order() {
    let todo_dir = setup();

    let Result { stdout, .. } = reexec(todo_dir, "listall", &[]);

    assert_eq!(
        stdout,
        r#"1 (A) Make peace between Cylons and humans +PeaceProject
2 (B) Report to <i>Admiral Adama</i> about FTL @CIC +Galactica\Repairs due:2013-05-24
4 (C) Upgrade jump drives with Cylon technology +Galactica\Repairs
3 2016-12-08 Feed Schrodinger's Cat 5 times due:2014-02-23
5 2016-12-12 +Galactica\Repairs Check hull integrity due:2016-12-12
6 Check for <b>DRADIS</b> contact @CIC
7 Check if http://google.com is available
8 Download code from <br/> https://github.com/QTodoTxt/QTodoTxt/archive/master.zip <br/>and give it a try!
9 Think about <u>future</u> t:2099-12-31
0 x 2016-02-21 (B) Seal ship's cracks with biomatter +Galactica\Repairs
--
TODO: 9 of 9 tasks show
DONE: 1 of 1 tasks show
total: 10 of 10 tasks show
"#
    );
}

#[test]
fn color() {
    let todo_dir = setup();

    let mut envs = HashMap::new();
    envs.insert("CLICOLOR_FORCE", "1");
    envs.insert("COLOR_CONTEXT", "(blue)");
    envs.insert("COLOR_DATE", "(light blue)");
    envs.insert("COLOR_DONE", "(gray)");
    envs.insert("COLOR_META", "(cyan)");
    envs.insert("COLOR_NUMBER", "(light grey)");
    envs.insert("COLOR_PROJECT", "(red)");
    envs.insert("PRI_A", "(yellow)");
    envs.insert("PRI_B", "(green)");
    envs.insert("PRI_C", "(blue)");

    let Result { stdout, .. } = reexec_env(todo_dir, "listall", &[], envs);

    let reset = "\x1B[0m";

    assert_eq!(
        stdout,
        format!(
            r#"(yellow)1 (A) Make peace between Cylons and humans (red)+PeaceProject{reset}{reset}
(green)2 (B) Report to <i>Admiral Adama</i> about FTL (blue)@CIC{reset} (red)+Galactica\Repairs{reset}(cyan) due:2013-05-24{reset}{reset}
(blue)4 (C) Upgrade jump drives with Cylon technology (red)+Galactica\Repairs{reset}{reset}
3 (light blue)2016-12-08 {reset}Feed Schrodinger's Cat (light grey)5{reset} times(cyan) due:2014-02-23{reset}
5 (light blue)2016-12-12 {reset}(red)+Galactica\Repairs{reset} Check hull integrity(cyan) due:2016-12-12{reset}
6 Check for <b>DRADIS</b> contact (blue)@CIC{reset}
7 Check if http://google.com is available
8 Download code from <br/> https://github.com/QTodoTxt/QTodoTxt/archive/master.zip <br/>and give it a try!
9 Think about <u>future</u>(cyan) t:2099-12-31{reset}
(gray)0 x (light blue)2016-02-21 {reset}(B) Seal ship's cracks with biomatter (red)+Galactica\Repairs{reset}{reset}
--
TODO: 9 of 9 tasks show
DONE: 1 of 1 tasks show
total: 10 of 10 tasks show
"#
        )
    );
}

#[test]
fn thresold_date() {
    let tomorrow = todo_txt::date::today().succ();

    let task = format!("Task in future t:{}", tomorrow.format("%Y-%m-%d"));
    let Result { todo_dir, .. } = exec("add", &[&task]);

    let result = reexec(todo_dir, "list", &[]);
    assert_eq!(result.stdout, "--\nTODO: 0 of 1 tasks show\n");
}

#[test]
fn listall() {
    let task = "new task 1\nnew task 2\nx done task";
    let Result { todo_dir, .. } = exec("addm", &[task]);
    let Result { todo_dir, .. } = reexec(todo_dir, "archive", &[]);

    let result = reexec(todo_dir, "listall", &[]);
    assert_eq!(
        result.stdout,
        r#"1 new task 1
2 new task 2
0 x done task
--
TODO: 2 of 2 tasks show
DONE: 1 of 1 tasks show
total: 3 of 3 tasks show
"#
    );

    let result = reexec(result.todo_dir, "listall", &["done"]);
    assert_eq!(
        result.stdout,
        r#"0 x done task
--
TODO: 0 of 2 tasks show
DONE: 1 of 1 tasks show
total: 1 of 3 tasks show
"#
    );
}

#[test]
fn listcon() {
    let todo_dir = setup();

    let result = reexec(todo_dir, "listcon", &[]);
    assert_eq!(result.stdout, "cic\n");
}

#[test]
fn listfile() {
    let task = "new task 1\nnew task 2\nx done task";
    let Result { todo_dir, .. } = exec("addm", &[task]);
    let Result { todo_dir, .. } = reexec(todo_dir, "archive", &[]);

    let result = reexec(todo_dir, "listfile", &["todo.txt"]);
    assert_eq!(
        result.stdout,
        r#"1 new task 1
2 new task 2
--
TODO: 2 of 2 tasks show
"#
    );

    let result = reexec(result.todo_dir, "listfile", &["done.txt"]);
    assert_eq!(
        result.stdout,
        r#"1 x done task
--
DONE: 1 of 1 tasks show
"#
    );
}

#[test]
fn listpri() {
    let todo_dir = setup();

    let result = reexec(todo_dir, "listpri", &[]);
    assert_eq!(
        result.stdout,
        r#"1 (A) Make peace between Cylons and humans +PeaceProject
2 (B) Report to <i>Admiral Adama</i> about FTL @CIC +Galactica\Repairs due:2013-05-24
4 (C) Upgrade jump drives with Cylon technology +Galactica\Repairs
--
TODO: 3 of 9 tasks show
"#
    );
}

#[test]
fn listproj() {
    let todo_dir = setup();

    let result = reexec(todo_dir, "listproj", &[]);
    assert_eq!(result.stdout, "peaceproject\ngalactica\\repairs\n");

    let result = reexec(result.todo_dir, "listproj", &["galactica"]);
    assert_eq!(result.stdout, "galactica\\repairs\n");
}

#[test]
fn r#move() {
    let task = "x task";
    let Result { todo_dir, .. } = exec("add", &[task]);
    let result = reexec(todo_dir, "archive", &[]);
    assert_eq!(result.done, "x task\n");

    let result = reexec(result.todo_dir, "move", &["1", "todo.txt", "done.txt"]);
    assert_eq!(result.todo, "x task\n");
    assert_eq!(
        result.stdout,
        format!(
            "1 x task\nTODO: 1 item moved from '{todo_dir}/done.txt' to '{todo_dir}/todo.txt'\n",
            todo_dir = result.todo_dir.display()
        )
    );
}

#[test]
#[cfg(feature = "extended")]
fn note() {
    let task = "new task";
    let Result { todo_dir, .. } = exec("add", &[task]);

    let result = reexec(todo_dir, "note", &["show", "1"]);
    assert_eq!(result.stdout, "TODO: Task 1 has no note.\n");

    let result = reexec(result.todo_dir, "note", &["add", "1"]);
    assert_eq!(result.stdout, "TODO: Note added to task 1\n");

    let result = reexec(result.todo_dir, "note", &["show", "1"]);
    assert_eq!(result.stdout, "");
}

#[test]
fn prepend() {
    let task = "task";
    let Result { todo_dir, .. } = exec("add", &[task]);

    let result = reexec(todo_dir, "prepend", &["1", "new"]);
    assert_eq!(result.todo, "new task\n");
    assert_eq!(result.stdout, "1 new task\n");
}

#[test]
fn pri() {
    let task = "new task 1\nnew task 2";
    let Result { todo_dir, .. } = exec("addm", &[task]);

    let result = reexec(todo_dir, "pri", &["2", "B"]);
    assert_eq!(result.todo, "new task 1\n(B) new task 2\n");
    assert_eq!(result.stdout, "2 (B) new task 2\nTODO: 2 prioritized (B)\n");
}

#[test]
#[cfg(feature = "extended")]
fn recurrence() {
    let task = "new task 1 t:2020-01-01 due:2020-02-02 rec:+1y";
    let Result { todo_dir, .. } = exec("add", &[task]);

    let result = reexec(todo_dir, "done", &["1"]);
    assert_eq!(result.done, "x new task 1 due:2020-02-02 t:2020-01-01 rec:+1y\n");
    assert_eq!(result.todo, "new task 1 due:2021-02-02 t:2021-01-01 rec:+1y\n");
}

#[test]
fn replace() {
    let task = "new task 1\nnew task 2";
    let Result { todo_dir, .. } = exec("addm", &[task]);

    let result = reexec(todo_dir, "replace", &["2", "another task 2"]);
    assert_eq!(result.todo, "new task 1\nanother task 2\n");
    assert_eq!(
        result.stdout,
        "2 new task 2\nTODO: Replaced task with:\n2 another task 2\n"
    );
}

#[test]
fn report() {
    let todo_dir = setup();

    let result = reexec(todo_dir, "report", &[]);
    let mut report = result.report.split(' ');
    assert_eq!(report.nth(1), Some("9"));
    assert_eq!(report.next(), Some("1\n"));
    assert_eq!(result.stdout, "TODO: Report file updated.\n");
}

fn setup() -> std::path::PathBuf {
    let tasks = include_str!("../examples/done.txt");
    let Result { todo_dir, .. } = exec("addm", &[tasks]);
    let Result { todo_dir, .. } = reexec(todo_dir, "archive", &[]);

    let tasks = include_str!("../examples/todo.txt");
    let Result { todo_dir, .. } = reexec(todo_dir, "addm", &[tasks]);

    todo_dir
}

fn exec(command: &str, args: &[&str]) -> Result {
    reexec(create_dir(), command, args)
}

fn create_dir() -> std::path::PathBuf {
    let todo_dir = mktemp::Temp::new_dir().unwrap().to_path_buf();
    std::fs::create_dir(&todo_dir).unwrap();

    todo_dir
}

fn reexec(todo_dir: std::path::PathBuf, command: &str, args: &[&str]) -> Result {
    let mut envs = HashMap::new();
    envs.insert("NO_COLOR", "true");

    reexec_env(todo_dir, command, args, envs)
}

fn reexec_env(
    todo_dir: std::path::PathBuf,
    command: &str,
    args: &[&str],
    envs: HashMap<&str, &str>,
) -> Result {
    let todo_file = todo_dir.join("todo.txt");
    let done_file = todo_dir.join("done.txt");
    let report_file = todo_dir.join("report.txt");

    let assert = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .arg(command)
        .args(args)
        .envs(envs)
        .env("TODOTXT_FORCE", "true")
        .env("TODO_DIR", &todo_dir)
        .env("TODO_ACTIONS_DIR", &todo_dir)
        .assert()
        .success();

    let to_string = |raw| String::from_utf8_lossy(raw).to_string();

    Result {
        todo_dir,
        stdout: to_string(&assert.get_output().stdout),
        todo: to_string(&std::fs::read(&todo_file).unwrap_or_default()),
        done: to_string(&std::fs::read(&done_file).unwrap_or_default()),
        report: to_string(&std::fs::read(&report_file).unwrap_or_default()),
    }
}
