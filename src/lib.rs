extern crate clap;
extern crate dirs;
extern crate libfund;

use std::error::Error;
use std::path::PathBuf;

use clap::ArgMatches;

pub struct Config {
    pub configdir: PathBuf,
    pub funddir: PathBuf,
    pub command: String,
    pub fund_name: Option<String>,
    pub transfer_name: Option<String>,
    pub amount: Option<i32>,
    pub goal: Option<i32>,
}

impl Config {
    pub fn new(matches: &ArgMatches) -> Result<Config, Box<Error + Send + Sync>> {
        let configdir = match dirs::config_dir() {
            Some(mut path) => {
                path.push(PathBuf::from(r"fund"));
                path
            }
            None => return Err(From::from("can't find config directory")),
        };
        let funddir = match dirs::data_dir() {
            Some(mut path) => {
                path.push(PathBuf::from(r"fund"));
                path
            }
            None => return Err(From::from("can't find data directory")),
        };

        let mut command = String::from(matches.subcommand().0);
        let mut fund_name = None;
        let mut amount = None;
        let mut goal = None;
        let mut transfer_name = None;

        match matches.subcommand() {
            ("new", Some(new_matches)) => {
                fund_name = new_matches.value_of("name");
                amount = new_matches.value_of("amount");
                goal = new_matches.value_of("goal");
            }
            ("deposit", Some(deposit_matches)) => {
                fund_name = deposit_matches.value_of("name");
                amount = deposit_matches.value_of("amount");
            }
            ("spend", Some(spend_matches)) => {
                fund_name = spend_matches.value_of("name");
                amount = spend_matches.value_of("amount");
            }
            ("info", Some(list_matches)) => {
                fund_name = list_matches.value_of("name");
            }
            ("transfer", Some(list_matches)) => {
                fund_name = list_matches.value_of("from_name");
                transfer_name = list_matches.value_of("to_name");
                amount = list_matches.value_of("amount");
            }
            ("", None) => command = String::from("info"),
            _ => unreachable!(),
        }

        let fund_name = fund_name.and_then(|x| Some(String::from(x)));
        let transfer_name = transfer_name.and_then(|x| Some(String::from(x)));
        let amount = amount.map_or(Ok(None), |x| x.replace(".", "").parse::<i32>().map(Some))?;
        let goal = goal.map_or(Ok(None), |x| x.replace(".", "").parse::<i32>().map(Some))?;

        Ok(Config {
            configdir,
            funddir,
            command,
            fund_name,
            transfer_name,
            amount,
            goal,
        })
    }
}

pub fn run(config: Config) -> Result<(), Box<Error + Send + Sync>> {
    let mut funds = libfund::FundManager::load(&config.funddir)?;

    match config.command.as_str() {
        "info" => match config.fund_name {
            Some(name) => funds.print_fund(&name)?,
            None => funds.print_all(),
        },
        "new" => match config.fund_name {
            Some(name) => {
                funds.add_fund(&name, config.amount, config.goal)?;
                funds.print_fund(&name)?;
            }
            None => return Err(From::from("can't create a new struct with no name")),
        },
        "spend" => match config.fund_name {
            Some(name) => match config.amount {
                Some(amount) => {
                    funds.get_fund_by_name(&name)?.spend(amount);
                    funds.print_fund(&name)?;
                }
                None => return Err(From::from("please supply an amount to spend")),
            },
            None => return Err(From::from("please supply a fund to spend from")),
        },
        "deposit" => match config.fund_name {
            Some(name) => match config.amount {
                Some(amount) => {
                    funds.get_fund_by_name(&name)?.deposit(amount);
                    funds.print_fund(&name)?;
                }
                None => return Err(From::from("please supply an amount to deposit")),
            },
            None => return Err(From::from("please supply a fund to deposit to")),
        },
        "transfer" => match config.fund_name {
            Some(name) => match config.transfer_name {
                Some(transfer_name) => match config.amount {
                    Some(amount) => {
                        funds.get_fund_by_name(&name)?.spend(amount);
                        funds.get_fund_by_name(&transfer_name)?.deposit(amount);
                        funds.print_fund(&name)?;
                        funds.print_fund(&transfer_name)?;
                    }
                    None => return Err(From::from("please supply an amount to transfer")),
                },
                None => return Err(From::from("please supply a fund to transfer to")),
            },
            None => return Err(From::from("please supply a fund to transfer from")),
        },
        _ => return Err(From::from("not a valid command")),
    }

    funds.save(&config.funddir)
}