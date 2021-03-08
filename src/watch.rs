use crate::{cmd, util::Fallible};

use clap::ArgMatches;
use notify::Watcher;
use std::{
    collections::HashSet, env, path::PathBuf, sync::mpsc::channel, thread::sleep, time::Duration,
};

pub fn watch(
    args: ArgMatches,
    watch_sources: HashSet<PathBuf>,
    watch_code: HashSet<PathBuf>,
) -> Fallible {
    trigger_on_change(&watch_sources, &watch_code, |paths| {
        let is_forward = paths
            .first()
            .map(|p| watch_sources.contains(p))
            .unwrap_or(true);

        println!(
            "{} changed. Re-building...",
            if is_forward { "Sources" } else { "Code" }
        );

        let curr_dir = env::current_dir()?;
        let (_config, _watch_forward, _watch_reverse) =
            cmd::run_with_args(&args, Some(!is_forward))?;
        env::set_current_dir(&curr_dir)?;

        Ok(())
    })?;

    Ok(())
}

/// Calls the closure when a book source file is changed, blocking indefinitely.
pub fn trigger_on_change<F>(
    watch_sources: &HashSet<PathBuf>,
    watch_code: &HashSet<PathBuf>,
    closure: F,
) -> Fallible
where
    F: Fn(Vec<PathBuf>) -> Fallible,
{
    use notify::DebouncedEvent::*;
    use notify::RecursiveMode::*;

    let (tx, rx) = channel();
    let mut source_watcher = notify::watcher(tx.clone(), Duration::from_secs(1))?;
    let mut code_watcher = notify::watcher(tx, Duration::from_secs(1))?;

    for path in watch_sources {
        source_watcher.watch(path, NonRecursive)?;
    }
    for path in watch_code {
        code_watcher.watch(path, NonRecursive)?;
    }

    loop {
        let first_event = rx.recv().unwrap();
        sleep(Duration::from_millis(50));
        let other_events = rx.try_iter();

        let all_events = std::iter::once(first_event).chain(other_events);

        let paths = all_events
            .filter_map(|event| match event {
                Create(path) | Write(path) | Remove(path) | Rename(_, path) => Some(path),
                _ => None,
            })
            .collect::<Vec<_>>();

        if !paths.is_empty() {
            closure(paths)?;
        }
    }
}
