use crate::{cmd, util::Fallible};

use clap::ArgMatches;
use notify::{DebouncedEvent, RecommendedWatcher, Watcher};
use std::sync::mpsc::{Receiver, Sender};
use std::time::Instant;
use std::{env, path::PathBuf, sync::mpsc::channel, time::Duration};

const COLLECT_EVENTS_MILLIS: u64 = 500;

#[derive(PartialEq, Clone)]
enum ChangeType {
    Sources,
    Code,
}

pub fn watch<I, J>(args: ArgMatches, mut watch_sources: I, mut watch_code: J) -> Fallible
where
    I: Iterator<Item = PathBuf>,
    J: Iterator<Item = PathBuf>,
{
    let rx_changes = trigger_on_change(&mut watch_sources, &mut watch_code)?;

    for change in rx_changes {
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
    }

    Ok(())
}

/// Calls the closure when a book source file is changed, blocking indefinitely.
fn trigger_on_change<I, J>(
    watch_sources: &mut I,
    watch_code: &mut J,
) -> Fallible<Receiver<ChangeType>>
where
    I: Iterator<Item = PathBuf>,
    J: Iterator<Item = PathBuf>,
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

    let (tx_changes, rx_changes) = channel();

    start_event_thread(
        source_watcher,
        rx_sources,
        tx_changes.clone(),
        ChangeType::Sources,
    );
    start_event_thread(code_watcher, rx_code, tx_changes, ChangeType::Code);

    Ok(rx_changes)
}

fn start_event_thread(
    watcher: RecommendedWatcher,
    in_channel: Receiver<DebouncedEvent>,
    out_channel: Sender<ChangeType>,
    event_type: ChangeType,
) {
    std::thread::spawn(move || {
        let _watcher = watcher;
        loop {
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
