use crate::{cmd, util::Fallible};

use clap::ArgMatches;
use notify::{DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::time::Instant;
use std::{env, path::PathBuf, sync::mpsc::channel, time::Duration};

const COLLECT_EVENTS_MILLIS: u64 = 500;

#[derive(PartialEq, Clone)]
enum ChangeType {
    Sources,
    Code,
}

pub fn watch(
    args: ArgMatches,
    watch_sources: impl Iterator<Item = PathBuf>,
    watch_code: impl Iterator<Item = PathBuf>,
    allow_reverse: bool,
) -> Fallible {
    println!("Watching for changes...");

    let mut watch_sources_old: HashSet<_> = watch_sources.collect();
    let mut watch_code_old: HashSet<_> = watch_code.collect();

    let suspend: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

    let (rx_changes, mut sw, mut cw) = trigger_on_change(
        watch_sources_old.iter(),
        watch_code_old.iter(),
        suspend.clone(),
    )?;

    for change in rx_changes {
        if allow_reverse || change == ChangeType::Sources {
            println!(
                "{} changed. Re-building...",
                if change == ChangeType::Sources {
                    "Sources"
                } else {
                    "Code"
                }
            );

            suspend.swap(true, Ordering::SeqCst);

            let curr_dir = env::current_dir()?;
            let (config, mut watch_sources_new, watch_code_new, _has_reverse) =
                cmd::run_with_args(&args, Some(change == ChangeType::Code))?;
            env::set_current_dir(&curr_dir)?;

            watch_sources_new.insert(config);

            update_watcher(&mut sw, &watch_sources_old, &watch_sources_new)?;
            update_watcher(&mut cw, &watch_code_old, &watch_code_new)?;

            suspend.swap(false, Ordering::SeqCst);

            watch_sources_old = watch_sources_new;
            watch_code_old = watch_code_new;
        }
    }

    Ok(())
}

fn update_watcher(
    watcher: &mut RecommendedWatcher,
    old_files: &HashSet<PathBuf>,
    new_files: &HashSet<PathBuf>,
) -> Fallible {
    for path in old_files.difference(new_files) {
        watcher.unwatch(path)?;
    }
    for path in new_files.difference(old_files) {
        watcher.watch(&path, RecursiveMode::NonRecursive)?;
    }
    Ok(())
}

/// Calls the closure when a book source file is changed, blocking indefinitely.
fn trigger_on_change<P>(
    watch_sources: impl Iterator<Item = P>,
    watch_code: impl Iterator<Item = P>,
    suspend: Arc<AtomicBool>,
) -> Fallible<(Receiver<ChangeType>, RecommendedWatcher, RecommendedWatcher)>
where
    P: AsRef<Path>,
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

    start_event_thread(
        rx_sources,
        tx_changes.clone(),
        ChangeType::Sources,
        suspend.clone(),
    );
    start_event_thread(rx_code, tx_changes, ChangeType::Code, suspend);

    Ok((rx_changes, source_watcher, code_watcher))
}

fn start_event_thread(
    in_channel: Receiver<DebouncedEvent>,
    out_channel: Sender<ChangeType>,
    event_type: ChangeType,
    suspend: Arc<AtomicBool>,
) {
    std::thread::spawn(move || loop {
        let mut send_event = false;

        let event = in_channel.recv().unwrap();
        if is_file_change(event) && !suspend.load(Ordering::SeqCst) {
            send_event = true;
        }

        let deadline = Instant::now() + Duration::from_millis(COLLECT_EVENTS_MILLIS);
        loop {
            let timeout = match deadline.checked_duration_since(Instant::now()) {
                None => break,
                Some(timeout) => timeout,
            };

            if let Ok(event) = in_channel.recv_timeout(timeout) {
                if is_file_change(event) && !suspend.load(Ordering::SeqCst) {
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
