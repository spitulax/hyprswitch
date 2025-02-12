use crate::cli::SwitchType;
use crate::daemon::gui::reload_desktop_maps;
use crate::daemon::submap::{activate_submap, deactivate_submap};
use crate::handle::{clear_recent_clients, collect_data, find_next, run_program, switch_to_active};
use crate::{Active, Command, Config, GUISend, GuiConfig, Share, UpdateCause, ACTIVE};
use anyhow::Context;
use std::ops::Deref;
use tracing::{info, trace, warn};

pub(crate) fn switch(share: &Share, command: Command, client_id: u8) -> anyhow::Result<()> {
    let (latest, send, receive) = share.deref();
    {
        let mut lock = latest.lock().expect("Failed to lock");
        let exec_len = lock.launcher.execs.len();
        if let Some(ref mut selected) = lock.launcher.selected {
            if exec_len == 0 {
                return Ok(());
            }
            *selected = if command.reverse {
                selected.saturating_sub(command.offset as u16)
            } else {
                (*selected + command.offset as u16).min((exec_len - 1) as u16)
            };
        } else {
            let active = find_next(
                &lock.simple_config.switch_type,
                command,
                &lock.hypr_data,
                &lock.active,
            )?;
            lock.active = active;
        }
        drop(lock);
    }

    trace!("Sending refresh to GUI");
    send.send_blocking((GUISend::Refresh, UpdateCause::Client(client_id)))
        .context("Unable to refresh the GUI")?;
    receive
        .recv_blocking()
        .context("Unable to receive GUI update")?;

    Ok(())
}

pub(crate) fn close(share: &Share, kill: bool, client_id: u8) -> anyhow::Result<()> {
    let (latest, send, receive) = share.deref();
    {
        let lock = latest.lock().expect("Failed to lock");
        if !kill {
            if let Some(selected) = lock.launcher.selected {
                if let Some((run, path, terminal)) = lock.launcher.execs.get(selected as usize) {
                    run_program(run, path, *terminal);
                } else {
                    warn!("Selected program (nr. {}) not found, killing", selected);
                }
            } else {
                switch_to_active(&lock.active, &lock.hypr_data)?;
            }
        } else {
            info!("Not executing switch on close, killing");
        }
        drop(lock);
    }
    deactivate_submap()?;
    *(ACTIVE
        .get()
        .expect("ACTIVE not set")
        .lock()
        .expect("Failed to lock")) = false;

    trace!("Sending refresh to GUI");
    send.send_blocking((GUISend::Hide, UpdateCause::Client(client_id)))
        .context("Unable to refresh the GUI")?;
    receive
        .recv_blocking()
        .context("Unable to receive GUI update")?;

    clear_recent_clients();
    reload_desktop_maps();
    Ok(())
}

pub(crate) fn init(
    share: &Share,
    config: Config,
    gui_config: GuiConfig,
    client_id: u8,
) -> anyhow::Result<()> {
    let (clients_data, active) = collect_data(config.clone())
        .with_context(|| format!("Failed to collect data with config {:?}", config.clone()))?;

    let active = match config.switch_type {
        SwitchType::Client => {
            if let Some(add) = active.0 {
                Active::Client(add)
            } else {
                Active::Unknown
            }
        }
        SwitchType::Workspace => {
            if let Some(ws) = active.1 {
                Active::Workspace(ws)
            } else {
                Active::Unknown
            }
        }
        SwitchType::Monitor => {
            if let Some(mon) = active.2 {
                Active::Monitor(mon)
            } else {
                Active::Unknown
            }
        }
    };

    let (latest, send, receive) = share.deref();
    {
        let mut lock = latest.lock().expect("Failed to lock");

        lock.active = active;
        lock.simple_config = config.clone();
        lock.gui_config = gui_config.clone();
        lock.hypr_data = clients_data;
        drop(lock);
    }
    activate_submap(gui_config.clone())?;
    *(ACTIVE
        .get()
        .expect("ACTIVE not set")
        .lock()
        .expect("Failed to lock")) = true;

    trace!("Sending refresh to GUI");
    send.send_blocking((GUISend::New, UpdateCause::Client(client_id)))
        .context("Unable to refresh the GUI")?;
    receive
        .recv_blocking()
        .context("Unable to receive GUI update")?;
    Ok(())
}
