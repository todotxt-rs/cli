#![warn(warnings)]

mod commands;
mod errors;
mod list;
mod opts;

use errors::*;
use list::*;
use opts::Opt;

#[cfg(not(feature = "extended"))]
pub(crate) type Task = todo_txt::Task;
#[cfg(feature = "extended")]
pub(crate) type Task = todo_txt::task::Extended;

fn main() -> Result {
    use clap::Parser;
    use envir::Serialize;
    use opts::Command::*;

    let Ok(mut opt) = Opt::try_parse() else {
        return help(&todo_txt::Config::from_env());
    };

    let config = todo_txt::Config::from(&opt);

    if opt.command.is_none() {
        let mut args = std::env::args_os().collect::<Vec<_>>();
        args.push(std::ffi::OsString::from(&config.default_action));

        opt.update_from(&args);
    }

    config.export();

    // @TODO create bash function for action

    if let Some(command) = opt.command {
        match command {
            Add(arg) => commands::add(&config, &arg),
            Addm(arg) => commands::addm(&config, &arg),
            Addto(arg) => commands::addto(&config, &arg),
            Append(arg) => commands::append(&config, &arg),
            Archive => commands::archive(&config),
            Deduplicate => commands::deduplicate(&config),
            Del(arg) => commands::del(&config, &arg),
            Delpri(arg) => commands::delpri(&config, &arg),
            Done(arg) => commands::done(&config, &arg),
            Flag(arg) => {
                if let Some(item) = arg.item {
                    commands::flag(&config, item)
                } else {
                    commands::listflag(&config)
                }
            }
            Help => help(&config),
            List(arg) => commands::list(&config, &arg),
            Listall(arg) => commands::listall(&config, &arg),
            Listaddons => commands::listaddons(&config),
            Listfile(arg) => commands::listfile(&config, &arg),
            Listcon(arg) => commands::listcon(&config, &arg),
            Listpri(arg) => commands::listpri(&config, &arg),
            Listproj(arg) => commands::listproj(&config, &arg),
            Move(arg) => commands::r#move(&config, &arg),
            #[cfg(feature = "extended")]
            Note(arg) => commands::note(&config, &arg),
            Prepend(arg) => commands::prepend(&config, &arg),
            Pri(arg) => commands::pri(&config, &arg),
            Replace(arg) => commands::replace(&config, &arg),
            Report => commands::report(&config),
            External(arg) => commands::external(&config, &arg),
        }
    } else {
        help(&config)
    }
}

fn help(config: &todo_txt::Config) -> Result {
    use clap::CommandFactory;

    let mut app = Opt::command();
    app.print_help()?;

    if config.verbose > 1 {
        println!(
            "
\x1B[0;33mENVIRONMENT:\x1B[0m
    \x1B[0;32mTODOTXT_AUTO_ARCHIVE\x1B[0m            is same as option -a (0)/-A (1)
    \x1B[0;32mTODOTXT_CFG_FILE=CONFIG_FILE\x1B[0m    is same as option -d CONFIG_FILE
    \x1B[0;32mTODOTXT_FORCE=1\x1B[0m                 is same as option -f
    \x1B[0;32mTODOTXT_PRESERVE_LINE_NUMBERS\x1B[0m   is same as option -n (0)/-N (1)
    \x1B[0;32mTODOTXT_PLAIN\x1B[0m                   is same as option -p (1)/-c (0)
    \x1B[0;32mTODOTXT_DATE_ON_ADD\x1B[0m             is same as option -t (1)/-T (0)
    \x1B[0;32mTODOTXT_PRIORITY_ON_ADD=pri\x1B[0m     default priority A-Z
    \x1B[0;32mTODOTXT_VERBOSE=1\x1B[0m               is same as option -v
    \x1B[0;32mTODOTXT_DISABLE_FILTER=1\x1B[0m        is same as option -x
    \x1B[0;32mTODOTXT_DEFAULT_ACTION=\"\"\x1B[0m       run this when called with no arguments
    \x1B[0;32mTODOTXT_SORT_COMMAND=\"sort ...\"\x1B[0m customize list output
    \x1B[0;32mTODOTXT_FINAL_FILTER=\"sed ...\"\x1B[0m  customize list after color, P@+ hiding
    \x1B[0;32mTODOTXT_SOURCEVAR=$DONE_FILE\x1B[0m    use another source for listcon, listproj
    \x1B[0;32mTODOTXT_SIGIL_BEFORE_PATTERN=\"\"\x1B[0m optionally allow chars preceding +p / @c
    \x1B[0;32mTODOTXT_SIGIL_VALID_PATTERN=.*\x1B[0m  tweak the allowed chars for +p and @c
    \x1B[0;32mTODOTXT_SIGIL_AFTER_PATTERN=\"\"\x1B[0m  optionally allow chars after +p / @c"
        );
    }

    println!("\n\x1B[0;33mADDONS:\x1B[0m");

    for entry in std::fs::read_dir(&config.action_dir)? {
        let command = match entry {
            Ok(entry) => entry.file_name().to_string_lossy().to_string(),
            Err(_) => continue,
        };

        let args = [command, "usage".to_string()];
        commands::external(config, &args).ok();
    }

    Ok(())
}
