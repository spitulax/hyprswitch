use std::fmt::Debug;
use std::path::PathBuf;
use std::str::FromStr;

use crate::handle::get_monitors;
use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug, Clone)]
#[command(
    author,
    version,
    about,
    long_about = "A CLI/GUI that allows switching between windows in Hyprland\nvisit https://github.com/H3rmt/hyprswitch/wiki/Examples to see Example configs"
)]
pub struct App {
    #[clap(flatten)]
    pub global_opts: GlobalOpts,

    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Args, Debug, Clone)]
pub struct GlobalOpts {
    /// Print the command that would be executed instead of executing it
    #[arg(short = 'd', long, global = true)]
    pub dry_run: bool,

    /// Increase the verbosity level (-v: info, -vv: debug, -vvv: trace)
    #[arg(short = 'v', action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// Initialize and start the Daemon
    Init {
        #[clap(flatten)]
        init_opts: InitOpts,
    },
    /// Used to send commands to the daemon (used in keymap that gets generated by gui)
    Dispatch {
        #[clap(flatten)]
        simple_opts: SimpleOpts,
    },
    /// Opens the GUI
    Gui {
        #[clap(flatten)]
        gui_conf: GuiConf,

        #[clap(flatten)]
        simple_config: SimpleConf,
    },
    /// Switch without using the GUI / Daemon (switches directly)
    Simple {
        #[clap(flatten)]
        simple_opts: SimpleOpts,

        #[clap(flatten)]
        simple_conf: SimpleConf,
    },
    /// Close the GUI, executes the command to switch window
    Close {
        /// Don't switch to the selected window, just close the GUI
        #[arg(long)]
        kill: bool,
    },
    /// Test command to debug finding icons for the GUI, doesn't interact with the Daemon
    Icon {
        /// The class of the window to find an icon for
        #[arg(long, default_value = "")]
        class: String,

        /// List all icons in the theme
        #[arg(long)]
        list: bool,

        /// List all desktop files
        #[arg(long)]
        desktop_files: bool,
    },
}

#[derive(Args, Debug, Clone)]
pub struct InitOpts {
    /// Specify a path to custom css file
    #[arg(long)]
    pub custom_css: Option<PathBuf>,

    /// Show the windows title instead of its class in Overview (fallback to class if title is empty)
    #[arg(long, default_value = "true", action = clap::ArgAction::Set, default_missing_value = "true", num_args=0..=1
    )]
    pub show_title: bool,

    /// Limit amount of workspaces in one row (overflows to next row)
    #[arg(long, default_value = "5", value_parser = clap::value_parser!(u8).range(1..))]
    pub workspaces_per_row: u8,

    /// The size factor (float) for the GUI (original_size / 30 * size_factor)
    #[arg(long, default_value = "6")]
    pub size_factor: f64,
}

#[derive(Args, Debug, Clone)]
pub struct SimpleConf {
    /// Include special workspaces (e.g., scratchpad)
    #[arg(long, default_value = "false", action = clap::ArgAction::Set, default_missing_value = "true", num_args=0..=1
    )]
    pub include_special_workspaces: bool,

    /// Sort all windows on every monitor like one contiguous workspace
    #[arg(long, default_value = "false", action = clap::ArgAction::Set, default_missing_value = "true", num_args=0..=1
    )]
    pub ignore_workspaces: bool,

    /// Sort all windows on matching workspaces on monitors like one big monitor
    #[arg(long, default_value = "false", action = clap::ArgAction::Set, default_missing_value = "true", num_args=0..=1
    )]
    pub ignore_monitors: bool,

    /// Only show/switch between windows that have the same class/type as the currently focused window
    #[arg(short = 's', long)]
    pub filter_same_class: bool,

    /// Only show/switch between windows that are on the same workspace as the currently focused window
    #[arg(short = 'w', long)]
    pub filter_current_workspace: bool,

    /// Only show/switch between windows that are on the same monitor as the currently focused window
    #[arg(short = 'm', long)]
    pub filter_current_monitor: bool,

    /// Sort windows by most recently focused
    #[arg(long)]
    pub sort_recent: bool,

    /// Switches to next / previous workspace / client / monitor
    #[arg(long, default_value_t, value_enum)]
    pub switch_type: SwitchType,
}

#[derive(ValueEnum, Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub enum SwitchType {
    #[default]
    Client,
    Workspace,
    Monitor,
}

#[derive(Args, Debug, Clone)]
pub struct SimpleOpts {
    /// Reverse the order of windows / switch backwards
    #[arg(short = 'r', long)]
    pub reverse: bool,

    /// Switch to a specific window offset (default 1)
    #[arg(short = 'o', long, default_value = "1", value_parser = clap::value_parser!(u8).range(1..)
    )]
    pub offset: u8,
}

#[derive(Args, Debug, Clone)]
pub struct GuiConf {
    /// The modifier key to used to open the GUI (e.g. shift, alt, ...)
    #[clap(long, value_enum)]
    pub mod_key: ModKeyInput,

    /// The key to used to open the GUI (e.g., tab, grave, ...)
    #[arg(long)]
    pub key: String,

    /// The key used for reverse switching. Format: reverse-key=mod=<MODIFIER> or reverse-key=key=<KEY> (e.g., --reverse-key=mod=shift, --reverse-key=key=grave)
    #[arg(long, value_parser = clap::value_parser!(ReverseKey), default_value = "mod=shift")]
    pub reverse_key: ReverseKey,

    /// How to close hyprswitch
    #[clap(long, default_value_t, value_enum)]
    pub close: CloseType,

    /// The maximum offset you can switch to with number keys and is shown in the GUI (pass 0 to disable the number keys and index)
    #[arg(long, default_value = "6", value_parser = clap::value_parser!(u8).range(0..=9))]
    pub max_switch_offset: u8,

    /// Hide the active window border in the GUI (also hides the border for selected workspace or monitor)
    #[arg(long, default_value = "false", action = clap::ArgAction::Set, default_missing_value = "true", num_args= 0..=1
    )]
    pub hide_active_window_border: bool,

    // pub tile_floating_windows: bool,
    /// Show the GUI only on this monitor(s) [default: display on all monitors]
    ///
    /// Example: `--monitors=HDMI-0,DP-1` / `--monitors=eDP-1`
    ///
    /// Available values: `hyprctl monitors -j | jq '.[].name'`
    #[arg(long, value_delimiter = ',', value_parser = clap::value_parser!(Monitors))]
    pub monitors: Option<Monitors>,

    /// Show all workspaces on all monitors [default: only show workspaces on the corresponding monitor]
    #[arg(long, default_value = "false", action = clap::ArgAction::Set, default_missing_value = "true", num_args=0..=1
    )]
    pub show_workspaces_on_all_monitors: bool,
}

#[derive(ValueEnum, Clone, Debug, Serialize, Deserialize)]
#[clap(rename_all = "snake_case")]
pub enum ModKeyInput {
    // = alt_l
    Alt,
    AltL,
    AltR,
    // = ctrl_;
    Ctrl,
    CtrlL,
    CtrlR,
    // = super_l
    Super,
    SuperL,
    SuperR,
    // = shift_l
    Shift,
    ShiftL,
    ShiftR,
}

#[derive(ValueEnum, Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModKey {
    AltL,
    AltR,
    CtrlL,
    CtrlR,
    #[default]
    SuperL,
    SuperR,
    ShiftL,
    ShiftR,
}

impl From<ModKeyInput> for ModKey {
    fn from(s: ModKeyInput) -> Self {
        match s {
            ModKeyInput::Alt | ModKeyInput::AltL => ModKey::AltL,
            ModKeyInput::AltR => ModKey::AltR,
            ModKeyInput::Ctrl | ModKeyInput::CtrlL => ModKey::CtrlL,
            ModKeyInput::CtrlR => ModKey::CtrlR,
            ModKeyInput::Super | ModKeyInput::SuperL => ModKey::SuperL,
            ModKeyInput::SuperR => ModKey::SuperR,
            ModKeyInput::Shift | ModKeyInput::ShiftL => ModKey::ShiftL,
            ModKeyInput::ShiftR => ModKey::ShiftR,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Monitors(pub Vec<String>);

impl FromStr for Monitors {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let available: Vec<hyprland::data::Monitor> = get_monitors();
        let mut vec = Vec::new();
        for monitor in s.split(',') {
            if let Some(m) = available.iter().find(|m| m.name == monitor) {
                vec.push(m.name.clone())
            } else {
                return Err(format!(
                    "{s} not found in {:?}",
                    available.iter().map(|a| a.name.clone()).collect::<Vec<_>>()
                ));
            }
        }
        Ok(Self(vec))
    }
}

#[derive(ValueEnum, Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub enum CloseType {
    #[default]
    /// Close when *pressing enter* or an index key (1, 2, 3, ...) or clicking on a window in GUI (or pressing escape)
    Default,
    /// Close when *releasing the mod key* (e.g., SUPER) or clicking on a window in GUI (or pressing escape)
    ModKeyRelease,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum ReverseKey {
    Mod(ModKey),
    Key(String),
}

impl Default for ReverseKey {
    fn default() -> Self {
        ReverseKey::Mod(ModKey::ShiftL)
    }
}

impl FromStr for ReverseKey {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('=').collect();
        if parts.len() != 2 {
            return Err(format!(
                "Invalid format for reverse: {} (use mod=<modifier> or key=<key>)",
                s
            ));
        }
        match (parts[0], parts[1]) {
            ("mod", value) => Ok(ReverseKey::Mod(ModKey::from(ModKeyInput::from_str(
                value, true,
            )?))),
            ("key", value) => Ok(ReverseKey::Key(value.to_string())),
            _ => Err(format!("Invalid format for reverse: {}", s)),
        }
    }
}
