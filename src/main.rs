mod cmd;
mod code;
mod compile;
mod config;
mod create;
mod files;
mod lock;
mod parse;
mod plugin;
mod print;
mod util;
mod watch;

extern crate yarner_lib;

use crate::util::Fallible;
use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use std::env;

fn main() {
    std::process::exit(match run() {
        Ok(()) => 0,
        Err(err) => {
            eprintln!("ERROR: {}", err);
            1
        }
    });
}

fn get_matches<'a>() -> ArgMatches<'a> {
    App::new("Yarner")
        .version(crate_version!())
        .about(r#"Literate Programming tool for Markdown
  https://github.com/mlange-42/yarner

The normal workflow is:
 1) Create a project with
    > yarner init
 2) Process the project by running
    > yarner"#)
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .value_name("path")
            .help("Sets the config file path")
            .takes_value(true)
            .default_value("Yarner.toml"))
        .arg(Arg::with_name("root")
            .long("root")
            .short("r")
            .value_name("path")
            .help("Root directory. Optional. Defaults to 'path -> root' from config file, or to the current directory.")
            .takes_value(true))
        .arg(Arg::with_name("docs")
            .short("d")
            .long("docs")
            .value_name("path")
            .help("Directory to output documentation files to. Optional. Defaults to 'path -> docs' from config file.")
            .takes_value(true))
        .arg(Arg::with_name("code")
            .short("o")
            .long("code")
            .value_name("path")
            .help("Directory to output code files to. Optional. Defaults to 'path -> code' from config file.")
            .takes_value(true))
        .arg(Arg::with_name("entrypoint")
            .short("e")
            .long("entrypoint")
            .value_name("name")
            .help("The named entrypoint to use when tangling code. Optional. Defaults to 'path -> entrypoint', or to the unnamed code block(s).")
            .takes_value(true))
        .arg(Arg::with_name("input")
            .help("The input source file(s) as glob pattern(s). Optional. Defaults to 'path -> files' from config file.")
            .value_name("FILES")
            .multiple(true)
            .index(1))
        .arg(Arg::with_name("clean")
            .long("clean")
            .short("C")
            .help("Produces clean code output, without block label comments.")
            .required(false)
            .takes_value(false))
        .arg(Arg::with_name("force")
            .long("force")
            .short("F")
            .help("Forces building, although it would result in overwriting changed files.")
            .required(false)
            .takes_value(false))
        .subcommand(SubCommand::with_name("init")
            .about("Creates a yarner project in the current directory")
        )
        .subcommand(SubCommand::with_name("reverse")
            .about("Reverse mode: play back code changes into source files")
        )
        .subcommand(SubCommand::with_name("watch")
            .about("Watch files and build project on changes")
        )
        .get_matches()
}

fn run() -> Fallible {
    let matches = get_matches();

    if matches.subcommand_matches("init").is_some() {
        create::create_new_project().map_err(|err| format!("Could not create project: {}", err))?;
        println!("Successfully created project.\nTo compile the project, run 'yarner' from here.",);
        return Ok(());
    }

    let curr_dir = env::current_dir()?;
    let (config, mut watch_forward, watch_reverse, has_reverse_conf) =
        cmd::run_with_args(&matches, None)?;
    env::set_current_dir(&curr_dir)?;

    if matches.subcommand_matches("watch").is_some() {
        watch_forward.insert(config);
        watch::watch(
            matches,
            watch_forward.into_iter(),
            watch_reverse.into_iter(),
            has_reverse_conf,
        )?;
    }

    Ok(())
}
