#[derive(clap::Parser)]
pub(crate) struct Opt {
    #[clap(subcommand)]
    pub command: Option<Command>,
    /// Hide context names in list output
    #[clap(short = '@')]
    pub hide_context: bool,
    /// Hide project names in list output
    #[clap(short = '+')]
    pub hide_project: bool,
    /// Color mode
    #[clap(short)]
    pub color: bool,
    /// Use a configuration file other than one of the defaults
    #[clap(short = 'd', default_value = "~/.todo/config")]
    pub config_file: String,
    /// Forces actions without confirmation or interactive input
    #[clap(short = 'f')]
    pub force: bool,
    /// Plain mode turns off colors
    #[clap(short)]
    pub plain_text: bool,
    /// Hide priority labels in list output.
    #[clap(short = 'P')]
    pub hide_priority: bool,
    /// Don't auto-archive tasks automatically on completion
    #[clap(short = 'a')]
    pub dont_auto_archive: bool,
    /// Don't preserve line numbers; automatically remove blank lines on task deletion
    #[clap(short = 'n')]
    pub dont_preserve_line_numbers: bool,
    /// Prepend the current date to a task automatically when it's added
    #[clap(short = 't')]
    pub append_current_date: bool,
    /// Verbose mode turns on confirmation messages
    #[clap(short, parse(from_occurrences))]
    pub verbose: u32,
    /// Displays version, license and credits
    #[clap(short = 'V')]
    pub version: bool,
    /// Disables TODOTXT_FINAL_FILTER
    #[clap(short = 'x')]
    pub disable_final_filter: bool,
}

#[derive(clap::Subcommand)]
pub(crate) enum Command {
    /// Adds THING I NEED TO DO to your todo.txt file on its own line.
    ///
    /// Project and context notation optional.
    /// Quotes optional.
    #[clap(alias = "a")]
    Add(Add),

    /// Adds FIRST THING I NEED TO DO to your todo.txt on its own line and
    /// Adds SECOND THING I NEED TO DO to you todo.txt on its own line.
    ///
    /// Project and context notation optional.
    Addm(Add),

    /// Adds a line of text to any file located in the todo.txt directory.
    ///
    /// For example, addto inbox.txt "decide about vacation"
    ///
    /// addto DEST "TEXT TO ADD"
    Addto(AddTo),

    /// Adds TEXT TO APPEND to the end of the task on line ITEM#.
    ///
    /// Quotes optional.
    #[clap(alias = "app")]
    Append(Append),

    /// Moves all done tasks from todo.txt to done.txt and removes blank lines.
    Archive,

    /// Removes duplicate lines from todo.txt.
    Deduplicate,

    /// Deletes the task on line ITEM# in todo.txt.
    ///
    /// If TERM specified, deletes only TERM from the task.
    #[clap(alias = "rm")]
    Del(Del),

    /// Deprioritizes (removes the priority) from the task(s) on line ITEM# in todo.txt.
    #[clap(alias = "dp")]
    Delpri(Item),

    /// Marks task(s) on line ITEM# as done in todo.txt.
    #[clap(alias = "do")]
    Done(Item),

    /// Display help about usage, options, built-in and add-on actions, or just the usage help for
    /// the passed ACTION(s).
    Help,

    /// Displays all tasks that contain TERM(s) sorted by priority with line numbers.
    ///
    /// Each task must match all TERM(s) (logical AND); to display tasks that contain any TERM
    /// (logical OR), use 'TERM1\|TERM2\|...' (with quotes), or TERM1\\\|TERM2 (unquoted). Hides
    /// all tasks that contain TERM(s) preceded by a minus sign (i.e. -TERM). TERM(s) are
    /// grep-style basic regular expressions; for literal matching, put a single backslash before
    /// any [ ] \ $ * . ^ and enclose the entire TERM in single quotes, or use double backslashes
    /// and extra shell-quoting.  If no TERM specified, lists entire todo.txt.
    #[clap(alias = "ls")]
    List(Filter),

    /// Displays all the lines in todo.txt AND done.txt that contain TERM(s) sorted by priority
    /// with line numbers.
    ///
    /// Hides all tasks that contain TERM(s) preceded by a minus sign (i.e. -TERM). If no TERM
    /// specified, lists entire todo.txt AND done.txt concatenated and sorted.
    #[clap(alias = "lsa")]
    Listall(Filter),

    /// Lists all added and overridden actions in the actions directory.
    Listaddons,

    /// Lists all the task contexts that start with the @ sign in todo.txt.
    ///
    /// If TERM specified, considers only tasks that contain TERM(s).
    #[clap(alias = "lsc")]
    Listcon(Filter),

    /// Displays all the lines in SRC file located in the todo.txt directory, sorted by priority
    /// with line numbers.
    ///
    /// If TERM specified, lists all lines that contain TERM(s) in SRC file. Hides all tasks that
    /// contain TERM(s) preceded by a minus sign (i.e. -TERM). Without any arguments, the names of
    /// all text files in the todo.txt directory are listed.
    #[clap(alias = "lf")]
    Listfile(ListFile),

    /// Displays all tasks prioritized PRIORITIES.
    ///
    /// PRIORITIES can be a single one (A) or a range (A-C). If no PRIORITIES specified, lists all
    /// prioritized tasks. If TERM specified, lists only prioritized tasks that contain TERM(s).
    /// Hides all tasks that contain TERM(s) preceded by a minus sign (i.e. -TERM).
    #[clap(alias = "lsp")]
    Listpri(ListPri),

    /// Lists all the projects (terms that start with a + sign) in todo.txt.
    ///
    /// If TERM specified, considers only tasks that contain TERM(s).
    #[clap(alias = "lsprj")]
    Listproj(Filter),

    /// Moves a line from source text file (SRC) to destination text file (DEST).
    ///
    /// Both source and destination file must be located in the directory defined in the
    /// configuration directory. When SRC is not defined it's by default todo.txt.
    #[clap(alias = "mv")]
    Move(Move),

    #[cfg(feature = "extended")]
    #[clap(subcommand)]
    Note(Note),

    /// Adds TEXT TO PREPEND to the beginning of the task on line ITEM#.
    ///
    /// Quotes optional.
    #[clap(alias = "prep")]
    Prepend(Append),

    /// Adds PRIORITY to task on line ITEM#.
    ///
    /// If the task is already prioritized, replaces current priority with new PRIORITY. PRIORITY
    /// must be a letter between A and Z.
    #[clap(alias = "p")]
    Pri(Pri),

    /// Replaces task on line ITEM# with UPDATED TODO.
    Replace(Replace),

    /// Adds the number of open tasks and done tasks to report.txt.
    Report,

    #[clap(external_subcommand)]
    External(Vec<String>),
}

#[derive(clap::Parser)]
pub(crate) struct Add {
    pub task: Option<String>,
}

#[derive(clap::Parser)]
pub(crate) struct AddTo {
    pub dest: String,
    #[clap(flatten)]
    pub add: Add,
}

#[derive(clap::Parser)]
pub(crate) struct Append {
    pub item: usize,
    #[clap(flatten)]
    pub add: Add,
}

#[derive(clap::Parser)]
pub(crate) struct Del {
    pub item: usize,
    #[clap(flatten)]
    pub filter: Filter,
}

#[derive(clap::Parser)]
pub(crate) struct Item {
    pub item: usize,
}

#[derive(clap::Parser)]
pub(crate) struct Filter {
    pub term: Option<String>,
}

#[derive(clap::Parser)]
pub(crate) struct ListFile {
    pub src: String,
    #[clap(flatten)]
    pub filter: Filter,
}

#[derive(clap::Parser)]
pub(crate) struct ListPri {
    pub priority: Option<char>,
    #[clap(flatten)]
    pub filter: Filter,
}

#[derive(clap::Parser)]
pub(crate) struct Move {
    pub item: usize,
    pub dest: String,
    #[clap(default_value = "todo.txt")]
    pub src: String,
}

#[cfg(feature = "extended")]
#[derive(clap::Subcommand)]
pub(crate) enum Note {
    Archive,
    #[clap(alias = "a")]
    Add(Item),
    #[clap(alias = "e")]
    Edit(Item),
    #[clap(alias = "s")]
    Show(Item),
}

#[derive(clap::Parser)]
pub(crate) struct Pri {
    pub item: usize,
    pub priority: char,
}

#[derive(clap::Parser)]
pub(crate) struct Replace {
    pub item: usize,
    pub text: Option<String>,
}
