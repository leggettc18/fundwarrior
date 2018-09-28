extern crate dirs;
extern crate clap;

use std::io::prelude::*;
use std::fs::OpenOptions;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use std::error::Error;

use clap::ArgMatches;

pub struct Config {
    pub configfile: PathBuf,
    pub fundfile: PathBuf,
    pub command: Option<String>,
    pub fund_name: Option<String>,
    pub amount: Option<i32>,
    pub goal: Option<i32>,
}

impl Config {
    pub fn new(matches: ArgMatches) -> Result<Config, Box<Error+Send+Sync>> {
        let mut configfile: PathBuf = dirs::config_dir().unwrap();
        configfile.push(PathBuf::from(r"fund/config"));
        let mut fundfile: PathBuf = dirs::home_dir().unwrap();
        fundfile.push(r".fund");

        let mut command = None;
        let mut fund_name = None;
        let mut amount = None;
        let mut goal = None;

        match matches.subcommand() {
            ("new", Some(new_matches)) => {
                command = Some(String::from("new"));
                fund_name = new_matches.value_of("name");
                amount = new_matches.value_of("amount");
                goal = new_matches.value_of("goal");
            },
            ("deposit", Some(deposit_matches)) => {
                command = Some(String::from("deposit"));
                fund_name = deposit_matches.value_of("name");
                amount = deposit_matches.value_of("amount");
            },
            ("spend", Some(spend_matches)) => {
                command = Some(String::from("spend"));
                fund_name = spend_matches.value_of("name");
                amount = spend_matches.value_of("amount");
            },
            ("list", Some(list_matches)) => {
                command = Some(String::from("list"));
                fund_name = list_matches.value_of("name");
            },
            ("", None) => command = Some(String::from("list")),
            _ => unreachable!(),
        }

        let fund_name = fund_name.map_or(None, |x| Some(String::from(x)));

        let amount = match amount {
            Some(x) => {
                Some(x.replace(".", "").parse::<i32>()?)
            },
            None => None,
        };

        let goal = match goal {
            Some(x) => {
                Some(x.replace(".", "").parse::<i32>()?)
            },
            None => None,
        };

        Ok(Config { configfile, fundfile, command, fund_name, amount, goal })
    }
}

pub fn run(config: Config) -> Result<(), Box<Error+Send+Sync>> {
    let command = config.command.clone();
    let fund_name = config.fund_name.clone();
    let amount = config.amount.clone();
    let goal = config.goal.clone();
    let mut funds = FundManager::load(&config)?;

    match command {
        None => funds.print_all(),
        Some(command) => {
            match command.as_ref() {
                "list" => {
                    match fund_name {
                        Some(name) => funds.get_fund_by_name(name)?.print_details(),
                        None => funds.print_all(),
                    }
                },
                "new" => {
                    match fund_name {
                        Some(name) => funds.add_fund(name, amount, goal)?,
                        None => return Err(From::from("can't create a new struct with no name")),
                    }
                },
                "spend" => {
                    match fund_name {
                        Some(name) => {
                            match amount {
                                Some(amount) => funds.get_fund_by_name(name)?.spend(amount),
                                None => return Err(From::from("please supply an amount to spend")),
                            }
                        }
                        None => return Err(From::from("please supply a fund to spend from")),
                    }
                },
                "deposit" => {
                    match fund_name {
                        Some(name) => {
                            match amount {
                                Some(amount) => funds.get_fund_by_name(name)?.deposit(amount),
                                None => return Err(From::from("please supply an amount to deposit")),
                            }
                        }
                        None => return Err(From::from("please supply a fund to deposit to")),
                    }
                },
                _ => return Err(From::from("not a valid command")),
            }
        }
    }

    funds.save(&config)
}

struct FundManager {
    funds: Vec<Fund>,
}

impl FundManager {
    pub fn load(config: &Config) -> Result<FundManager, Box<Error+Send+Sync>> {
        let fundfile = config.fundfile.clone();
        let file = OpenOptions::new().read(true).write(true).create(true).open(fundfile)?;
        let mut funds: Vec<Fund> = Vec::new();
        let buf_reader = BufReader::new(file);

        for line in buf_reader.lines() {
            let line = line?;
            let fund_info: Vec<&str> = line.split_terminator(":").collect();
            let name: String = match fund_info[0].parse() {
                Ok(name) => name,
                Err(e) => {
                    return Err(From::from(format!("while parsing {:?}: {}", config.fundfile, e)))
                }
            };
            let amount: i32 = match fund_info[1].parse() {
                Ok(amount) => amount,
                Err(e) => {
                    return Err(From::from(format!("while parsing {:?}: {}", config.fundfile, e)))
                }
            };
            let goal: i32 = match fund_info[2].parse() {
                Ok(goal) => goal,
                Err(e) => {
                    return Err(From::from(format!("while parsing {:?}: {}", config.fundfile, e)))
                }
            };
            funds.push( Fund{ name, amount, goal });
        }

        Ok(FundManager{ funds })
    }

    pub fn save(self, config: &Config) -> Result<(), Box<Error+Send+Sync>> {
        let fundfile = config.fundfile.clone();
        let file = OpenOptions::new().write(true).create(true).open(fundfile)?;
        let mut buf_writer = BufWriter::new(file);
        for fund in self.funds {
            let string = format!("{}:{}:{}\n", fund.name, fund.amount, fund.goal);
            buf_writer.write(string.as_bytes())?;
        }
        Ok(())
    }

    pub fn get_fund_by_name(&mut self, name: String) -> Result<&mut Fund, &'static str> {
        let query = Fund::new(name, None, None)?;
        let index = self.funds.iter().position(|x| x.eq(&query));
        match index {
            Some(x) => Ok(&mut self.funds[x]),
            None => Err("can't find a fund with that name"),
        }
    }    

    pub fn print_all(&self) {
        println!("{:10} | {:11} | {:11}", "Name", "Amount", "Goal");
        for fund in self.funds.iter() {
            fund.print_details();
        }
        for fund in self.funds.iter() {
            fund.print_goal_status();
        }
    }

    pub fn add_fund(&mut self, name: String, amount: Option<i32>, goal: Option<i32>) -> Result<(), Box<Error+Send+Sync>> {
        self.funds.push(Fund::new(name, amount, goal)?);
        Ok(())
    }
}

#[derive(Debug)]
struct Fund {
    name: String,
    amount: i32,
    goal: i32,
}

impl Fund {
    pub fn new(name: String, amount: Option<i32>, goal: Option<i32>) -> Result<Fund, &'static str> {
        if String::from(name.trim()).is_empty() {
            return Err("fund name cannot be blank");
        } else {
            Ok(Fund {
                name,
                amount: amount.unwrap_or(0),
                goal: goal.unwrap_or(0),
            })
        }
    }

    pub fn spend(&mut self, amount: i32) {
        self.amount -= amount;
    }

    pub fn deposit(&mut self, amount: i32) {
        self.amount += amount;
    }

    pub fn transfer_to(&mut self, fund: &mut Fund, amount: i32) {
        self.spend(amount);
        fund.deposit(amount);
    }

    //pub fn get_by_name(funds: &mut Vec<Fund>, name: String) -> Result<&mut Fund, &'static str> {
    //    let query = Fund::new(name, None, None)?;
    //    let index = funds.iter().position(|x| x.eq(&query));
    //    match index {
    //        Some(x) => Ok(&mut funds[x]),
    //        None => Err("can't find a fund with that name"),
    //    }
    //}

    pub fn print_goal_status(&self) {
        if self.amount >= self.goal {
            println!("Your goal of {} has been acheived for fund {}", self.goal, self.name);
        } else {
            println!("Fund {} is {} away from its {} goal", self.name, 
                Fund::display_dollars(self.goal - self.amount), Fund::display_dollars(self.goal));
        }
    }

    pub fn print_details(&self) {
        println!("{:10} | {:<10} | {:<10}", self.name, 
            Fund::display_dollars(self.amount), Fund::display_dollars(self.goal));
    }

    fn display_dollars(amount: i32) -> String {
        let mut amount = amount.to_string();
        while amount.len() < 3 {
            amount.insert(0, '0');
        }
        let (dollars, cents) = amount.split_at(amount.len()-2);
        String::from(format!("${}.{}", dollars, cents))
    }

    //pub fn print_all(funds: &Vec<Fund>) {
    //    println!("{:10} | {:11} | {:11}", "Name", "Amount", "Goal");
    //    for fund in funds {
    //        fund.print_details();
    //    }
    //    for fund in funds {
    //        fund.print_goal_status();
    //    }
    //}

    //pub fn save(funds: &Vec<Fund>, config: &Config) -> Result<(), Box<Error+Send+Sync>> {
    //    let fundfile = config.fundfile.clone();
    //    let file = OpenOptions::new().write(true).create(true).open(fundfile)?;
    //    let mut buf_writer = BufWriter::new(file);
    //    for fund in funds {
    //        let string = format!("{}:{:.2}:{:.2}\r\n", fund.name, fund.amount, fund.goal);
    //        buf_writer.write(string.as_bytes())?;
    //    }
    //   Ok(())
    //}

    //pub fn load(config: &Config) -> Result<Vec<Fund>, Box<Error+Send+Sync>> {
    //    let fundfile = config.fundfile.clone();
    //    let file = OpenOptions::new().read(true).write(true).create(true).open(fundfile)?;
    //    let mut funds: Vec<Fund> = Vec::new();
    //    let buf_reader = BufReader::new(file);
    //    for line in buf_reader.lines() {
    //        let line = line?;
    //        let fund_info: Vec<&str> = line.split_terminator(":").collect();
    //        let name: String = match fund_info[0].parse() {
    //            Ok(name) => name,
    //            Err(e) => {
    //                return Err(From::from(format!("while parsing {:?}: {}", config.fundfile, e)))
    //            }
    //        };
    //        let amount: i32 = match fund_info[1].parse() {
    //            Ok(amount) => amount,
    //            Err(e) => {
    //                return Err(From::from(format!("while parsing {:?}: {}", config.fundfile, e)))
    //            }
    //        };
    //        let goal: i32 = match fund_info[2].parse() {
    //            Ok(goal) => goal,
    //            Err(e) => {
    //                return Err(From::from(format!("while parsing {:?}: {}", config.fundfile, e)))
    //            }
    //        };
    //        funds.push( Fund{ name, amount, goal });
    //    }
    //
    //    Ok(funds)
    //}

}

impl PartialEq for Fund {
    fn eq (&self, x: &Fund) -> bool {
        x.name == self.name
    }
}



#[cfg(test)]
mod tests {
    use super::{Fund};

    #[test]
    fn create_fund() {
        let fund = Fund::new(String::from("Test"), None, None).unwrap();
        assert_eq!(fund.name, "Test");
        assert_eq!(fund.amount, 0);
        assert_eq!(fund.goal, 0);
        let fund_with_args = Fund::new(String::from("Test"), Some(500), Some(1000)).unwrap();
        assert_eq!(fund_with_args.name, "Test");
        assert_eq!(fund_with_args.amount, 500);
        assert_eq!(fund_with_args.goal, 1000);
    }

    #[test]
    fn fund_err_on_blank_name() {
        let fund = Fund::new(String::from(" "), None, None);
        assert!(fund.is_err());
    }

    #[test]
    fn fund_deposit() {
        let mut fund = Fund::new(String::from("Test"), Some(500), Some(1000)).unwrap();
        fund.deposit(500);
        assert_eq!(fund.amount, 1000);
    }
    #[test]
    fn fund_spend() {
        let mut fund = Fund::new(String::from("Test"), Some(500), Some(1000)).unwrap();
        fund.spend(250);
        assert_eq!(fund.amount, 250);
    }
}