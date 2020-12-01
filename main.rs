mod memory;
mod message;
mod program;

mod runner;
mod saver;

use std::path::PathBuf;

use anyhow::anyhow;

use message::Message;
use program::{Program, ProgramId};
use saver::State;
use clap::{Arg, App, SubCommand};

fn path() -> PathBuf {
    let mut path = PathBuf::from(directories::BaseDirs::new().expect("base path exist").home_dir());
    path.push(".gear");
    std::fs::create_dir_all(path.clone()).expect("Faield to create user dir");

    path.push("state.dat");

    path
}

fn main() -> Result<(), anyhow::Error> {

    let matches = App::new("Gear Test")
        .version("1.0")
        .author("Nikolay Volf <nikvolf@gmail.com>")
        .about("Run gear in cli")
        .subcommand(
            SubCommand::with_name("message")
                .about("Send message into the saved state")
                .arg(Arg::with_name("target").help("Message target").required(true))
                .arg(Arg::with_name("text").help("Message data in utf-8"))
            )
        .subcommand(
            SubCommand::with_name("add")
                .about("Add program to the saved state")
                .arg(Arg::with_name("id").help("Program Id").required(true))
                .arg(Arg::with_name("path").help("Path to the executable").required(true))
            )
        .get_matches();

    println!("Working state: {}", path().to_string_lossy());

    if let Some(matches) = matches.subcommand_matches("add") {
        let file_name = matches.value_of("path").expect("required above, cannot fail");
        let program_id: ProgramId = matches
            .value_of("id").expect("required above, cannot fail")
            .parse::<u64>().expect("should be a number")
            .into();

        let mut state = saver::load_from_file(path());
        let mut runner = state.into_runner();

        runner.update_program_code(
            program_id,
            std::fs::read(file_name)?.into(),
        );

        let state = State::from_runner(runner);
        saver::save_to_file(path(), &state);

    } else if let Some(matches) = matches.subcommand_matches("message") {

        let mut state = saver::load_from_file(path());

        let message_text = matches.value_of("text").map(|x| x.to_string()).unwrap_or_default();
        let message_target = matches
            .value_of("target").expect("required above, cannot fail")
            .parse::<u64>().expect("should be a number")
            .into();

        state.queued_messages.push(
            Message { source: 0.into(), dest: message_target, payload: message_text.into_bytes().into() }
        );

        let mut runner = state.into_runner();

        runner.run_next()?;

        let state = State::from_runner(runner);
        saver::save_to_file(path(), &state);

    }


    Ok(())
}
