use std::collections::HashMap;

#[derive(Clone, envir::Deserialize, envir::Serialize)]
#[envir(prefix = "COLOR_")]
pub struct Colors {
    #[envir(default)]
    pub context: crate::Color,
    #[envir(default)]
    pub done: crate::Color,
    #[envir(default)]
    pub date: crate::Color,
    #[envir(default)]
    pub meta: crate::Color,
    #[envir(default)]
    pub number: crate::Color,
    #[envir(default)]
    pub project: crate::Color,
    #[envir(load_with = "priorities_load", export_with = "priorities_export")]
    priorities: HashMap<String, crate::Color>,
}

fn priorities_load(env: &HashMap<String, String>) -> envir::Result<HashMap<String, crate::Color>> {
    let mut priorities = HashMap::new();
    priorities.insert("PRI_A".to_string(), "yellow".into());
    priorities.insert("PRI_B".to_string(), "green".into());
    priorities.insert("PRI_C".to_string(), "bright blue".into());
    priorities.insert("PRI_X".to_string(), "white".into());

    for (k, v) in env {
        if k.starts_with("PRI_") {
            priorities.insert(k.clone(), v.parse().unwrap_or_default());
        }
    }

    Ok(priorities)
}

fn priorities_export(priorities: &HashMap<String, crate::Color>) -> HashMap<String, String> {
    priorities
        .iter()
        .map(|(k, v)| (k.clone(), v.to_string()))
        .collect()
}

impl Colors {
    pub fn for_pri(&self, pri: &todo_txt::Priority) -> crate::Color {
        self.priorities
            .get(&format!("PRI_{pri}"))
            .unwrap_or_else(|| self.priorities.get("PRI_X").unwrap())
            .clone()
    }
}

#[derive(envir::Deserialize, envir::Serialize)]
#[envir(prefix = "TODOTXT_")]
pub struct Config {
    #[envir(noprefix, name = "TODO_ACTIONS_DIR", default = "${HOME}/.todo/actions")]
    pub action_dir: String,
    #[envir(default = "true")]
    pub auto_archive: bool,
    #[envir(nested)]
    pub colors: Colors,
    #[envir(default)]
    pub date_on_add: bool,
    #[envir(default = "help")]
    pub default_action: String,
    #[envir(default)]
    pub disable_filter: bool,
    #[envir(default = "cat")]
    pub final_filter: String,
    #[envir(default)]
    pub force: bool,
    #[envir(
        noprefix,
        name = "TODO_NOTE_ARCHIVE",
        default = "${TODO_DIR}/notes/archive.txt"
    )]
    pub note_archive: String,
    #[envir(noprefix, name = "TODO_NOTES_DIR", default = "${TODO_DIR}/notes")]
    pub notes_dir: String,
    #[envir(default)]
    pub plain: bool,
    #[envir(default = "true")]
    pub preserve_line_numbers: bool,
    pub priority_on_add: Option<char>,
    #[envir(default = "env LC_COLLATE=C sort -f -k2")]
    pub sort_command: String,
    #[envir(noprefix)]
    pub todo_dir: String,
    #[envir(noprefix, default = "${TODO_DIR}/todo.txt")]
    pub todo_file: String,
    #[envir(noprefix, default = "${TODO_DIR}/done.txt")]
    pub done_file: String,
    #[envir(noprefix, default = "${TODO_DIR}/report.txt")]
    pub report_file: String,
    #[envir(default = "1")]
    pub verbose: u8,
}

impl Config {
    pub(crate) fn from_opt(opt: &crate::Opt) -> Self {
        let mut config = Self::from_env();

        config.auto_archive |= !opt.dont_auto_archive;
        config.date_on_add |= opt.append_current_date;
        config.disable_filter |= opt.disable_final_filter;
        config.force |= opt.force;
        config.plain |= opt.plain_text;
        config.preserve_line_numbers |= !opt.dont_preserve_line_numbers;
        config.verbose |= opt.verbose;

        config
    }

    pub fn from_env() -> Self {
        Self::load_env();

        envir::from_env().unwrap()
    }

    pub fn load_env() {
        std::env::set_var("NONE", "");
        std::env::set_var("BLACK", "\x1B[0;30m");
        std::env::set_var("RED", "\x1B[0;31m");
        std::env::set_var("GREEN", "\x1B[0;32m");
        std::env::set_var("BROWN", "\x1B[0;33m");
        std::env::set_var("BLUE", "\x1B[0;34m");
        std::env::set_var("PURPLE", "\x1B[0;35m");
        std::env::set_var("CYAN", "\x1B[0;36m");
        std::env::set_var("LIGHT_GREY", "\x1B[0;37m");
        std::env::set_var("DARK_GREY", "\x1B[1;30m");
        std::env::set_var("LIGHT_RED", "\x1B[1;31m");
        std::env::set_var("LIGHT_GREEN", "\x1B[1;32m");
        std::env::set_var("YELLOW", "\x1B[1;33m");
        std::env::set_var("LIGHT_BLUE", "\x1B[1;3");
        std::env::set_var("LIGHT_PURPLE", "\x1B[1;35m");
        std::env::set_var("LIGHT_CYAN", "\x1B[1;36m");
        std::env::set_var("WHITE", "\x1B[1;37m");
        std::env::set_var("DEFAULT", "\x1B[0m");

        Self::load_config_file();
    }

    #[cfg(not(debug_assertions))]
    fn load_config_file() {
        let home = std::env::var("HOME").unwrap();

        let configs: Vec<Box<dyn Fn() -> String>> = vec![
            Box::new(|| format!("{home}/.todo/config")),
            Box::new(|| format!("{home}/todo.cfg")),
            Box::new(|| format!("{home}/.todo.cfg")),
            Box::new(|| format!("{home}/.config/todo/config")),
            Box::new(|| {
                let filename = std::env::args().next().unwrap();
                let mut path = std::path::PathBuf::from(filename);
                path.pop();

                format!("{}/todo.cfg", path.display())
            }),
            Box::new(|| {
                std::env::var("TODOTXT_GLOBAL_CFG_FILE")
                    .unwrap_or_else(|_| "/etc/todo/config".to_string())
            }),
        ];

        for config in configs {
            if dotenv::from_path(config()).is_ok() {
                break;
            }
        }
    }

    #[cfg(debug_assertions)]
    fn load_config_file() {
        dotenv::dotenv().ok();
    }
}
