extern crate clap;
extern crate dirs;
extern crate libfund;

use std::error::Error;
use std::path::PathBuf;
use std::io;

use clap::ArgMatches;

pub struct Config {
    pub configdir: PathBuf,
    pub fundfile: PathBuf,
    pub command: String,
    pub fund_name: Option<String>,
    pub transfer_name: Option<String>,
    pub field: Option<String>,
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
        let mut fundfile = match dirs::data_dir() {
            Some(data_dir) => data_dir,
            None => return Err(From::from("can't use this directory"))
        };
        fundfile.push("fund/fund");

        let mut command = String::from(matches.subcommand().0);
        let mut fund_name = None;
        let mut amount = None;
        let mut goal = None;
        let mut transfer_name = None;
        let mut field = None;

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
            ("rename", Some(list_matches)) => {
                fund_name = list_matches.value_of("old_name");
                transfer_name = list_matches.value_of("new_name");
            }
            ("set", Some(list_matches)) => {
                fund_name = list_matches.value_of("name");
                amount = list_matches.value_of("amount");
                field = list_matches.value_of("field");
            }
            ("", None) => command = String::from("info"),
            _ => unreachable!(),
        }

        let fund_name = fund_name.and_then(|x| Some(String::from(x)));
        let transfer_name = transfer_name.and_then(|x| Some(String::from(x)));
        let field = field.and_then(|x| Some(String::from(x)));
        let amount = amount.map_or(Ok(None), |x| x.replace(".", "").parse::<i32>().map(Some))?;
        let goal = goal.map_or(Ok(None), |x| x.replace(".", "").parse::<i32>().map(Some))?;

        Ok(Config {
            configdir,
            fundfile,
            command,
            fund_name,
            transfer_name,
            field,
            amount,
            goal,
        })
    }
}

pub fn run(config: Config) -> Result<(), libfund::FundManagerError> {
    let mut funds = libfund::FundManager::load(&config.fundfile)?;

    match config.command.as_str() {
        "info" => match config.fund_name {
            Some(name) => funds.print_fund(&name)?,
            None => funds.print_all(),
        },
        "new" => match config.fund_name {
            Some(name) => {
                let mut fund = libfund::Fund::new();
                if let Some(amount) = config.amount {
                    fund.with_amount(amount);
                }
                if let Some(goal) = config.goal {
                    fund.with_goal(goal);
                }
                let fund = fund.build();
                funds.add_fund(&name, fund)?;
                funds.print_fund(&name)?;
            }
            None => return Err(From::from(io::Error::new(io::ErrorKind::InvalidInput, "can't create a new struct with no name"))),
        },
        "spend" => match config.fund_name {
            Some(name) => match config.amount {
                Some(amount) => {
                    funds.fund_mut(&name)?.spend(amount);
                    funds.print_fund(&name)?;
                }
                None => return Err(From::from(io::Error::new(io::ErrorKind::InvalidInput, "please supply an amount to spend"))),
            },
            None => return Err(From::from(io::Error::new(io::ErrorKind::InvalidInput, "please supply a fund to spend from"))),
        },
        "deposit" => match config.fund_name {
            Some(name) => match config.amount {
                Some(amount) => {
                    funds.fund_mut(&name)?.deposit(amount);
                    funds.print_fund(&name)?;
                }
                None => return Err(From::from(io::Error::new(io::ErrorKind::InvalidInput, "please supply an amount to deposit"))),
            },
            None => return Err(From::from(io::Error::new(io::ErrorKind::InvalidInput, "please supply a fund to deposit to"))),
        },
        "transfer" => match config.fund_name {
            Some(name) => match config.transfer_name {
                Some(transfer_name) => match config.amount {
                    Some(amount) => {
                        funds.fund_mut(&name)?.spend(amount);
                        funds.fund_mut(&transfer_name)?.deposit(amount);
                        funds.print_fund(&name)?;
                        funds.print_fund(&transfer_name)?;
                    }
                    None => return Err(From::from(io::Error::new(io::ErrorKind::InvalidInput, "please supply an amount to transfer"))),
                },
                None => return Err(From::from(io::Error::new(io::ErrorKind::InvalidInput, "please supply a fund to transfer to"))),
            },
            None => return Err(From::from(io::Error::new(io::ErrorKind::InvalidInput, "please supply a fund to transfer from"))),
        },
        "rename" => match config.fund_name {
            Some(name) => match config.transfer_name {
                Some(transfer_name) => {
                    funds.rename(&name, &transfer_name)?;
                    funds.print_fund(&transfer_name)?;
                },
                None => return Err(From::from(io::Error::new(io::ErrorKind::InvalidInput, "please supply a new unique name"))),
            },
            None => return Err(From::from(io::Error::new(io::ErrorKind::InvalidInput, "please supply the name of the fund to rename"))),
        },
        "set" => match config.fund_name {
            Some(name) => match config.amount {
                Some(amount) => match config.field {
                    Some(field) => {
                        match field.as_str() {
                            "amount" => funds.fund_mut(&name)?.amount = amount,
                            "goal" => funds.fund_mut(&name)?.goal = amount,
                            _ => return Err(From::from(io::Error::new(io::ErrorKind::InvalidInput, "invalid field name"))),
                        };
                        funds.print_fund(&name)?;
                    },
                    None => return Err(From::from(io::Error::new(io::ErrorKind::InvalidInput, "please provide a field name"))),
                },
                None => return Err(From::from(io::Error::new(io::ErrorKind::InvalidInput, "please provide an amount"))),
            },
            None => return Err(From::from(io::Error::new(io::ErrorKind::InvalidInput, "please provide a fund name"))),
        },
        _ => return Err(From::from(io::Error::new(io::ErrorKind::InvalidInput, "not a valid command"))),
    }
    funds.save(&config.fundfile)
}
