use crate::{cmd, util::Fallible};

use clap::ArgMatches;
use log::info;
use notify::{RawEvent, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::path::Path;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::{Receiver, Sender},
    Arc,
};
use std::{env, path::PathBuf, sync::mpsc::channel, time::Duration};

const COLLECT_EVENTS: Duration = Duration::from_millis(1000);

#[derive(PartialEq, Clone, Copy)]
enum ChangeType {
    Sources,
    Code,
}

pub fn watch(
    args: ArgMatches,
    watch_sources: impl Iterator<Item = PathBuf>,
    watch_code: impl Iterator<Item = PathBuf>,
) -> Fallible {
    info!("Watching for changes...");

    let mut watch_sources_old: HashSet<_> = watch_sources.collect();
    let mut watch_code_old: HashSet<_> = watch_code.collect();

    let suspend: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

    let (rx_changes, mut sw, mut cw) = trigger_on_change(
        watch_sources_old.iter(),
        watch_code_old.iter(),
        suspend.clone(),
    )?;

    for change in rx_changes {
        info!(
            "{} changed. Re-building...",
            if change == ChangeType::Sources {
                "Sources"
            } else {
                "Code"
            }
        );

        suspend.store(true, Ordering::SeqCst);

        let curr_dir = env::current_dir()?;
        let (config, mut watch_sources_new, watch_code_new) =
            cmd::run_with_args(&args, Some(change == ChangeType::Code), false)?;
        env::set_current_dir(&curr_dir)?;

        watch_sources_new.insert(config);

        update_watcher(&mut sw, &watch_sources_old, &watch_sources_new)?;
        update_watcher(&mut cw, &watch_code_old, &watch_code_new)?;

        suspend.store(false, Ordering::SeqCst);

        watch_sources_old = watch_sources_new;
        watch_code_old = watch_code_new;
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
    let mut source_watcher = notify::raw_watcher(tx_sources)?;
    let (tx_code, rx_code) = channel();
    let mut code_watcher = notify::raw_watcher(tx_code)?;

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
    in_channel: Receiver<RawEvent>,
    out_channel: Sender<ChangeType>,
    event_type: ChangeType,
    suspend: Arc<AtomicBool>,
) {
    std::thread::spawn(move || loop {
        in_channel.recv().unwrap();
        if suspend.load(Ordering::SeqCst) {
            continue;
        }

        while in_channel.recv_timeout(COLLECT_EVENTS).is_ok() {}
        if suspend.load(Ordering::SeqCst) {
            continue;
        }

        out_channel.send(event_type).unwrap();
    });
}
