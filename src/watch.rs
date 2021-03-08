use crate::{cmd, util::Fallible};

use clap::ArgMatches;
use notify::{DebouncedEvent, Watcher};
use std::time::Instant;
use std::{collections::HashSet, env, path::PathBuf, sync::mpsc::channel, time::Duration};

const COLLECT_EVENTS_MILLIS: u64 = 500;

#[derive(PartialEq)]
pub enum ChangeType {
    Sources,
    Code,
}

pub fn watch(
    args: ArgMatches,
    watch_sources: HashSet<PathBuf>,
    watch_code: HashSet<PathBuf>,
) -> Fallible {
    trigger_on_change(&watch_sources, &watch_code, |change| {
        println!(
            "{} changed. Re-building...",
            if change == ChangeType::Sources {
                "Sources"
            } else {
                "Code"
            }
        );

        if change == ChangeType::Sources {
            let curr_dir = env::current_dir()?;
            let (_config, _watch_forward, _watch_reverse) =
                cmd::run_with_args(&args, Some(change == ChangeType::Code))?;
            env::set_current_dir(&curr_dir)?;
        }

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
    F: Fn(ChangeType) -> Fallible,
{
    use notify::RecursiveMode::*;

    let (tx_sources, rx_sources) = channel();
    let mut source_watcher = notify::watcher(tx_sources, Duration::from_secs(1))?;
    let (tx_code, rx_code) = channel();
    let mut code_watcher = notify::watcher(tx_code, Duration::from_secs(1))?;

    for path in watch_sources {
        source_watcher.watch(path, NonRecursive)?;
    }
    for path in watch_code {
        code_watcher.watch(path, NonRecursive)?;
    }

    let (tx_changes_sources, rx_changes) = channel();
    let tx_changes_code = tx_changes_sources.clone();
    std::thread::spawn(move || loop {
        let deadline = Instant::now() + Duration::from_millis(COLLECT_EVENTS_MILLIS);
        let mut send_event = false;
        loop {
            let timeout = deadline.saturating_duration_since(Instant::now());
            if timeout.as_nanos() == 0 {
                break;
            }

            if let Ok(event) = rx_sources.recv_timeout(timeout) {
                if is_file_change(event) {
                    send_event = true;
                }
            }
        }
        if send_event {
            tx_changes_sources.send(ChangeType::Sources).unwrap();
        }
    });

    std::thread::spawn(move || loop {
        let deadline = Instant::now() + Duration::from_millis(COLLECT_EVENTS_MILLIS);
        let mut send_event = false;
        loop {
            let timeout = deadline.saturating_duration_since(Instant::now());
            if timeout.as_nanos() == 0 {
                break;
            }

            if let Ok(event) = rx_code.recv_timeout(timeout) {
                if is_file_change(event) {
                    send_event = true;
                }
            }
        }
        if send_event {
            tx_changes_code.send(ChangeType::Code).unwrap();
        }
    });

    loop {
        let event = rx_changes.recv().unwrap();
        closure(event)?;
    }
}

fn is_file_change(event: DebouncedEvent) -> bool {
    matches!(
        event,
        DebouncedEvent::Create(_)
            | DebouncedEvent::Write(_)
            | DebouncedEvent::Remove(_)
            | DebouncedEvent::Rename(_, _)
    )
}
