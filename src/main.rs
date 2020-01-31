extern crate swayipc;
use swayipc::{Connection, EventType, Fallible};

extern crate swaywsr;

extern crate exitfailure;
use exitfailure::ExitFailure;

#[macro_use]
extern crate clap;
use clap::{App, Arg};

fn main() -> Result<(), ExitFailure> {
    let matches = App::new("swaywsr - sway workspace renamer")
       .version(crate_version!())
       .author(crate_authors!(",\n"))
        .arg(Arg::with_name("icons")
            .long("icons")
            .help("Sets icons to be used")
            .possible_values(&["awesome"])
            .takes_value(true))
        .arg(Arg::with_name("no-names")
            .long("no-names")
            .help("Set to no to display only icons (if available)"))
        .arg(Arg::with_name("config")
            .long("config")
            .short("c")
            .help("Path to toml config file")
            .takes_value(true))
       .get_matches();

    let icons = matches.value_of("icons").unwrap_or("");
    let no_names = matches.is_present("no-names");
    let options = match matches.value_of("config") {
        Some(filename) => {
            let file_config = match swaywsr::config::read_toml_config(filename) {
                Ok(config) => config,
                Err(e) => panic!("Could not parse config file\n {}", e)
            };
            swaywsr::Options {
                icons: file_config.icons.into_iter().chain(swaywsr::icons::get_icons(&icons)).collect(),
                aliases: file_config.aliases,
                general: file_config.general,
                names: !no_names
            }
        },
        None => {
            swaywsr::Options {
                icons: swaywsr::icons::get_icons(&icons),
                aliases: swaywsr::config::EMPTY_MAP.clone(),
                general: swaywsr::config::EMPTY_MAP.clone(),
                names: !no_names
            }
        }
    };

    let mut listener = I3EventListener::connect()?;
    let subs = [EventType::Window, EventType::Wokspace];
    listener.subscribe(&subs)?;

    for event in Connection::new()?.subscribe(&subs)? {
        println!("{:?}\n", event?)
    }
    // swaywsr::update_tree(&x_conn, &mut i3_conn, &options)?;

    // for event in listener.listen() {
    //     match event? {
    //         Event::WindowEvent(e) => {
    //             if let Err(error) = swaywsr::handle_window_event(&e, &x_conn, &mut i3_conn, &options) {
    //                 eprintln!("handle_window_event error: {}", error);
    //             }
    //         }
    //         Event::WorkspaceEvent(e) => {
    //             if let Err(error) = swaywsr::handle_ws_event(&e, &x_conn, &mut i3_conn, &options) {
    //                 eprintln!("handle_ws_event error: {}", error);
    //             }
    //         }
    //         _ => {}
    //     }
    // }

    Ok(())
}
