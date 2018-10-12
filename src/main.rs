extern crate clap;
extern crate fund;
extern crate libfund;

use std::process;

use clap::{App, Arg, SubCommand};
use fund::Config;

fn main() {
    let matches = App::new("fundwarrior")
                        .version("0.8.1")
                        .author("Christopher Leggett <leggettc18@gmail.com>")
                        .about("Simple CLI Money Management")
                        .arg(Arg::with_name("fundfile")
                            .short("f")
                            .long("fundfile")
                            .value_name("FILE")
                            .help("Sets a custom fund file")
                            .takes_value(true))
                        .arg(Arg::with_name("verbose")
                            .short("v")
                            .long("verbose")
                            .help("Enables verbose output"))
                        .subcommand(SubCommand::with_name("new")
                            .about("Creates a new fund")
                            .arg(Arg::with_name("name")
                                .help("The name of the fund to create")
                                .required(true))
                            .arg(Arg::with_name("amount")
                                .help("The amount to start the fund with")
                                .required(false))
                            .arg(Arg::with_name("goal")
                                .help("The amount you want this fund to have in the future")
                                .required(false)))
                        .subcommand(SubCommand::with_name("deposit")
                            .about("Deposit money into a fund")
                            .arg(Arg::with_name("name")
                                .help("The name of the fund you are depositing into")
                                .required(true))
                            .arg(Arg::with_name("amount")
                                .help("The amount you wish to deposit")
                                .required(true)))
                        .subcommand(SubCommand::with_name("spend")
                            .about("Spend money from a fund")
                            .arg(Arg::with_name("name")
                                .help("The name of the fund you are spending from")
                                .required(true))
                            .arg(Arg::with_name("amount")
                                .help("The amount you are spending")
                                .required(true)))
                        .subcommand(SubCommand::with_name("info")
                            .about("View fund information")
                            .arg(Arg::with_name("name")
                                .help("The name of the fund you wish to view. If absent, all funds will be printed.")
                                .required(false)))
                        .subcommand(SubCommand::with_name("transfer")
                            .about("Transfer money between funds")
                            .arg(Arg::with_name("from_name")
                                .help("The name of the fund you wish to transfer money out of")
                                .required(true))
                            .arg(Arg::with_name("to_name")
                                .help("The name of the fund you wish to transfer money to")
                                .required(true))
                            .arg(Arg::with_name("amount")
                                .help("The amount you wish to transfer")
                                .required(true)))
                        .subcommand(SubCommand::with_name("rename")
                            .about("Rename a fund")
                            .arg(Arg::with_name("old_name")
                                .help("The name of the fund you wish to rename")
                                .required(true))
                            .arg(Arg::with_name("new_name")
                                .help("The unique name you wish to give the fund")
                                .required(true)))
                        .subcommand(SubCommand::with_name("set")
                            .about("Set the amount or goal of a fund")
                            .arg(Arg::with_name("name")
                                .help("The name of the fund you wish to update")
                                .required(true))
                            .arg(Arg::with_name("field")
                                .help("The name of the field you wish to change (either amount or goal)")
                                .required(true))
                            .arg(Arg::with_name("amount")
                                .help("The amount you wish to change the specied field to")
                                .required(true)))
                        .get_matches();

    let config = Config::new(&matches);

    match config {
        Err(e) => {
            eprintln!("Error parsing arguments: {}", e);
            process::exit(1);
        }
        Ok(config) => {
            if let Err(e) = fund::run(config) {
                eprintln!("Application error: {}", e);
                process::exit(1);
            }
        }
    }
}
