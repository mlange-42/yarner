use crate::{cmd, util::Fallible};

use clap::ArgMatches;
use notify::{DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Instant;
use std::{env, path::PathBuf, sync::mpsc::channel, time::Duration};

const COLLECT_EVENTS_MILLIS: u64 = 500;

#[derive(PartialEq, Clone)]
enum ChangeType {
    Sources,
    Code,
}

pub fn watch<I, J>(
    args: ArgMatches,
    watch_sources: I,
    watch_code: J,
    allow_reverse: bool,
) -> Fallible
where
    I: Iterator<Item = PathBuf>,
    J: Iterator<Item = PathBuf>,
{
    println!("Watching for changes...");

    let mut watch_sources_old: HashSet<_> = watch_sources.collect();
    let mut watch_code_old: HashSet<_> = watch_code.collect();

    trigger_on_change(
        watch_sources_old.clone(),
        watch_code_old.clone(),
        |change, sw, cw| {
            if allow_reverse || change == ChangeType::Sources {
                println!(
                    "{} changed. Re-building...",
                    if change == ChangeType::Sources {
                        "Sources"
                    } else {
                        "Code"
                    }
                );

                let curr_dir = env::current_dir()?;
                let (config, mut watch_sources_new, watch_code_new, _has_reverse) =
                    cmd::run_with_args(&args, Some(change == ChangeType::Code))?;
                env::set_current_dir(&curr_dir)?;

                watch_sources_new.insert(config);

                for path in &watch_sources_old {
                    sw.unwatch(path)?;
                }
                for path in &watch_code_old {
                    cw.unwatch(path)?;
                }

                watch_sources_old.clear();
                watch_code_old.clear();
                for path in watch_sources_new {
                    sw.watch(&path, RecursiveMode::NonRecursive)?;
                    watch_sources_old.insert(path);
                }
                for path in watch_code_new {
                    cw.watch(&path, RecursiveMode::NonRecursive)?;
                    watch_code_old.insert(path);
                }
            }
            Ok(())
        },
    )?;

    Ok(())
}

/// Calls the closure when a book source file is changed, blocking indefinitely.
fn trigger_on_change<F>(
    watch_sources: HashSet<PathBuf>,
    watch_code: HashSet<PathBuf>,
    mut closure: F,
) -> Fallible<Receiver<ChangeType>>
where
    F: FnMut(ChangeType, &mut RecommendedWatcher, &mut RecommendedWatcher) -> Fallible,
{
    let (tx_sources, rx_sources) = channel();
    let mut source_watcher = notify::watcher(tx_sources, Duration::from_secs(1))?;
    let (tx_code, rx_code) = channel();
    let mut code_watcher = notify::watcher(tx_code, Duration::from_secs(1))?;

    for path in watch_sources {
        source_watcher.watch(path, RecursiveMode::NonRecursive)?;
    }
    for path in watch_code {
        code_watcher.watch(path, RecursiveMode::NonRecursive)?;
    }

    let (tx_changes, rx_changes) = channel();

    start_event_thread(rx_sources, tx_changes.clone(), ChangeType::Sources);
    start_event_thread(rx_code, tx_changes, ChangeType::Code);

    loop {
        let event = rx_changes.recv().unwrap();
        closure(event, &mut source_watcher, &mut code_watcher)?;
    }
}

fn start_event_thread(
    in_channel: Receiver<DebouncedEvent>,
    out_channel: Sender<ChangeType>,
    event_type: ChangeType,
) {
    std::thread::spawn(move || loop {
        let mut send_event = false;

        let event = in_channel.recv().unwrap();
        if is_file_change(event) {
            send_event = true;
        }

        let deadline = Instant::now() + Duration::from_millis(COLLECT_EVENTS_MILLIS);
        loop {
            let timeout = match deadline.checked_duration_since(Instant::now()) {
                None => break,
                Some(timeout) => timeout,
            };

            if let Ok(event) = in_channel.recv_timeout(timeout) {
                if is_file_change(event) {
                    send_event = true;
                }
            }
        }

        if send_event {
            out_channel.send(event_type.clone()).unwrap();
        }
    });
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
