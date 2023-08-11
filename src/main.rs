use anyhow::Context;
use clap::Parser;
use std::path::PathBuf;
use swayipc::{Connection, Event, EventType};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Set to no to display only icons (if available)
    #[arg(short, long)]
    #[clap(value_parser(["awesome"]))]
    icons: Option<String>,
    /// Set to no to display only icons (if available)
    #[arg(short, long, default_value_t = false)]
    no_names: bool,
    /// Path to toml config file
    #[arg(short, long)]
    config: Option<PathBuf>,
    /// Remove duplicate entries in workspace
    #[arg(short, long, default_value_t = false)]
    remove_duplicates: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let config_path = cli
        .config
        .unwrap_or(swaywsr::config::xdg_config_home().join("swaywsr/config.toml"));
    let file_config = swaywsr::config::read_toml_config(&config_path)
        .with_context(|| format!("Could not parse config file at {}", config_path.display()))?;

    let icons = cli.icons.unwrap_or("".to_string());
    let mut config = swaywsr::Config {
        icons: swaywsr::icons::get_icons(&icons)
            .into_iter()
            .chain(file_config.icons)
            .collect(),
        aliases: file_config.aliases,
        general: file_config.general,
        options: file_config.options,
    };

    if cli.no_names {
        config.options.insert("no_names".to_string(), cli.no_names);
    }
    if cli.remove_duplicates {
        config
            .options
            .insert("remove_duplicates".to_string(), cli.remove_duplicates);
    }

    let subs = [EventType::Window, EventType::Workspace];
    let connection = Connection::new()?;
    let mut command_connection = Connection::new()?;

    swaywsr::update_tree(&mut command_connection, &config)?;

    for event in connection.subscribe(subs)? {
        match event? {
            Event::Window(e) => {
                if let Err(error) =
                    swaywsr::handle_window_event(&e, &mut command_connection, &config)
                {
                    eprintln!("handle_window_event error: {}", error);
                }
            }
            Event::Workspace(e) => {
                if let Err(error) =
                    swaywsr::handle_workspace_event(&e, &mut command_connection, &config)
                {
                    eprintln!("handle_workspace_event error: {}", error);
                }
            }
            _ => {}
        }
    }

    Ok(())
}
