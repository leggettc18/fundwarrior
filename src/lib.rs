extern crate dirs;

use std::io::prelude::*;
use std::fs::OpenOptions;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use std::error::Error;

pub struct Config {
    pub configfile: PathBuf,
    pub fundfile: PathBuf,
    pub command: Option<String>,
    pub fund_name: Option<String>,
    pub amount: Option<f64>,
    pub goal: Option<f64>,
}

impl Config {
    pub fn new(args: &[String]) -> Config {
        let mut configfile: PathBuf = dirs::config_dir().unwrap();
        configfile.push(PathBuf::from(r"fund/config"));
        let mut fundfile: PathBuf = dirs::home_dir().unwrap();
        fundfile.push(r".fundrc");

        let command = if args.len() > 1 { 
            Some(args[1].clone())
        } else {
            None
        };
        let fund_name = if args.len() > 2 { 
            Some(args[2].clone())
        } else {
            None
        };
        let amount = if args.len() > 3 { 
            Some(args[3].clone().parse().unwrap())
        } else {
            None
        };
        let goal = if args.len() > 4 {
            Some(args[4].clone().parse().unwrap())
        } else {
            None
        };

        Config { configfile, fundfile, command, fund_name, amount, goal }
    }
}

pub fn run(config: Config) -> Result<(), Box<Error+Send+Sync>> {
    let command = config.command.clone();
    let fund_name = config.fund_name.clone();
    let amount = config.amount.clone();
    let goal = config.goal.clone();
    let mut funds: Vec<Fund> = Fund::load(&config)?;

    match command {
        None => Fund::print_all(&funds),
        Some(command) => {
            match command.as_ref() {
                "list" => {
                    match fund_name {
                        Some(name) => Fund::get_by_name(&mut funds, name)?.print_details(),
                        None => Fund::print_all(&funds),
                    }
                },
                "new" => {
                    match fund_name {
                        Some(name) => funds.push(Fund::new(name, amount, goal)?),
                        None => return Err(From::from("can't create a new struct with no name")),
                    }
                },
                "spend" => {
                    match fund_name {
                        Some(name) => {
                            match amount {
                                Some(amount) => Fund::get_by_name(&mut funds, name)?.spend(amount),
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
                                Some(amount) => Fund::get_by_name(&mut funds, name)?.deposit(amount),
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

    Fund::save(&funds, &config)
}

struct Fund {
    name: String,
    amount: f64,
    goal: f64,
}

impl Fund {
    pub fn new(name: String, amount: Option<f64>, goal: Option<f64>) -> Result<Fund, &'static str> {
        if String::from(name.trim()).is_empty() {
            return Err("fund name cannot be blank");
        } else {
            Ok(Fund {
                name,
                amount: amount.unwrap_or(0.0),
                goal: goal.unwrap_or(0.0),
            })
        }
    }

    pub fn spend(&mut self, amount: f64) {
        self.amount -= amount;
    }

    pub fn deposit(&mut self, amount: f64) {
        self.amount += amount;
    }

    pub fn transfer_to(&mut self, fund: &mut Fund, amount: f64) {
        self.spend(amount);
        fund.deposit(amount);
    }

    pub fn get_by_name(funds: &mut Vec<Fund>, name: String) -> Result<&mut Fund, &'static str> {
        for fund in funds {
            if fund.name == name {
                return Ok(fund);
            }
        }
        Err("can't find a fund with that name")
    }

    pub fn print_goal_status(&self) {
        if self.amount >= self.goal {
            println!("Your goal of ${} has been acheived for fund {}", self.goal, self.name);
        } else {
            println!("Fund {} is ${} away from its ${} goal", self.name, self.goal - self.amount, self.goal);
        }
    }

    pub fn print_details(&self) {
        println!("{:10} | ${:6.2} | ${:6.2}", self.name, self.amount, self.goal);
    }

    pub fn print_all(funds: &Vec<Fund>) {
        println!("{:10} | {:6.2} | {:6.2}", "Name", "Amount", "Goal");
        for fund in funds {
            fund.print_details();
        }
        for fund in funds {
            fund.print_goal_status();
        }
    }

    pub fn save(funds: &Vec<Fund>, config: &Config) -> Result<(), Box<Error+Send+Sync>> {
        let fundfile = config.fundfile.clone();
        let file = OpenOptions::new().write(true).create(true).open(fundfile)?;
        let mut buf_writer = BufWriter::new(file);
        for fund in funds {
            let string = format!("{}:{:.2}:{:.2}\r\n", fund.name, fund.amount, fund.goal);
            buf_writer.write(string.as_bytes())?;
        }
        Ok(())
    }

    pub fn load(config: &Config) -> std::io::Result<Vec<Fund>> {
        let fundfile = config.fundfile.clone();
        let file = OpenOptions::new().read(true).open(fundfile)?;
        let mut funds: Vec<Fund> = Vec::new();
        let buf_reader = BufReader::new(file);

        for line in buf_reader.lines() {
            let line = line.unwrap();
            let fund_info: Vec<&str> = line.split_terminator(":").collect();
            let name: String = fund_info[0].parse().unwrap();
            let amount: f64 = fund_info[1].parse().unwrap();
            let goal: f64 = fund_info[2].parse().unwrap();
            funds.push( Fund{ name, amount, goal });
        }

        Ok(funds)
    }

}



#[cfg(test)]
mod tests {
    use super::{Fund};

    #[test]
    fn create_fund() {
        let fund = Fund::new(String::from("Test"), None, None).unwrap();
        assert_eq!(fund.name, "Test");
        assert_eq!(fund.amount, 0.0);
        assert_eq!(fund.goal, 0.0);
        let fund_with_args = Fund::new(String::from("Test"), Some(50.0), Some(100.0)).unwrap();
        assert_eq!(fund_with_args.name, "Test");
        assert_eq!(fund_with_args.amount, 50.0);
        assert_eq!(fund_with_args.goal, 100.0);
    }

    #[test]
    fn fund_err_on_blank_name() {
        let fund = Fund::new(String::from(" "), None, None);
        assert!(fund.is_err());
    }

    #[test]
    fn fund_deposit() {
        let mut fund = Fund::new(String::from("Test"), Some(50.0), Some(100.0)).unwrap();
        fund.deposit(50.0);
        assert_eq!(fund.amount, 100.0);
    }
    #[test]
    fn fund_spend() {
        let mut fund = Fund::new(String::from("Test"), Some(50.0), Some(100.0)).unwrap();
        fund.spend(25.0);
        assert_eq!(fund.amount, 25.0);
    }
}