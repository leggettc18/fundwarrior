use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fs;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

pub struct FundManager {
    funds: HashMap<String, Fund>,
}

impl FundManager {
    pub fn load(funddir: &PathBuf) -> Result<FundManager, Box<Error + Send + Sync>> {
        fs::create_dir_all(funddir)?;
        let mut fundfile = funddir.to_owned();
        fundfile.push(r"fund");
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&fundfile)?;
        let mut funds: HashMap<String, Fund> = HashMap::new();
        let buf_reader = BufReader::new(file);

        for line in buf_reader.lines() {
            let line = line?;
            let fund_info: Vec<&str> = line.split_terminator(':').collect();
            if fund_info.len() < 3 {
                return Err(From::from(format!("{:?} is invalid", fundfile)));
            }
            let name = match fund_info[0].parse() {
                Ok(name) => name,
                Err(e) => return Err(From::from(format!("while parsing {:?}: {}", fundfile, e))),
            };
            let amount: i32 = match fund_info[1].parse() {
                Ok(amount) => amount,
                Err(e) => return Err(From::from(format!("while parsing {:?}: {}", fundfile, e))),
            };
            let goal: i32 = match fund_info[2].parse() {
                Ok(goal) => goal,
                Err(e) => return Err(From::from(format!("while parsing {:?}: {}", fundfile, e))),
            };
            funds.insert(name, Fund { amount, goal });
        }

        Ok(FundManager { funds })
    }

    pub fn save(self, funddir: &PathBuf) -> Result<(), Box<Error + Send + Sync>> {
        fs::create_dir_all(funddir)?;
        let mut fundfile = funddir.to_owned();
        fundfile.push(r"fund");
        let file = OpenOptions::new().write(true).create(true).open(fundfile)?;
        let mut buf_writer = BufWriter::new(file);
        for fund in self.funds {
            let string = format!("{}:{}:{}\n", fund.0, fund.1.amount, fund.1.goal);
            buf_writer.write_all(string.as_bytes())?;
        }
        Ok(())
    }

    pub fn get_fund_by_name(&mut self, name: &str) -> Result<&mut Fund, &'static str> {
        match self.funds.get_mut(name) {
            Some(fund) => Ok(fund),
            None => Err("cannot find the fund"),
        }
    }

    pub fn print_fund(&mut self, name: &str) -> Result<(), Box<Error + Send + Sync>> {
        let fund = self.get_fund_by_name(name)?;
        let mut name = String::from(name);
        name.push(':');
        println!("{:<10} {}", name, fund);
        Ok(())
    }

    pub fn print_all(&self) {
        for fund in &self.funds {
            let mut name = fund.0.to_owned();
            name.push(':');
            println!("{:>10} {}", name, fund.1)
        }
    }

    pub fn add_fund(
        &mut self,
        name: &str,
        amount: Option<i32>,
        goal: Option<i32>,
    ) -> Result<(), Box<Error + Send + Sync>> {
        if self.funds.contains_key(name) {
            return Err(From::from(format!(
                "fund '{}' already exists. Please choose a different name",
                name
            )));
        }
        self.funds
            .insert(String::from(name), Fund::new(amount, goal));
        Ok(())
    }
}

#[derive(Debug)]
pub struct Fund {
    amount: i32,
    goal: i32,
}

impl Fund {
    pub fn new(amount: Option<i32>, goal: Option<i32>) -> Fund {
        Fund {
            amount: amount.unwrap_or(0),
            goal: goal.unwrap_or(0),
        }
    }

    pub fn spend(&mut self, amount: i32) {
        self.amount -= amount;
    }

    pub fn deposit(&mut self, amount: i32) {
        self.amount += amount;
    }

    fn display_dollars(amount: i32) -> String {
        let mut amount = amount.to_string();
        while amount.len() < 3 {
            amount.insert(0, '0');
        }
        let (dollars, cents) = amount.split_at(amount.len() - 2);
        format!("${}.{}", dollars, cents)
    }
}

impl fmt::Display for Fund {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:^8} / {:<8} -- {} away from goal",
            Fund::display_dollars(self.amount),
            Fund::display_dollars(self.goal),
            Fund::display_dollars(self.goal - self.amount)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Fund;

    #[test]
    fn create_fund() {
        let fund = Fund::new(None, None);
        assert_eq!(fund.amount, 0);
        assert_eq!(fund.goal, 0);
        let fund_with_args = Fund::new(Some(500), Some(1000));
        assert_eq!(fund_with_args.amount, 500);
        assert_eq!(fund_with_args.goal, 1000);
    }

    #[test]
    fn fund_deposit() {
        let mut fund = Fund::new(Some(500), Some(1000));
        fund.deposit(500);
        assert_eq!(fund.amount, 1000);
    }
    #[test]
    fn fund_spend() {
        let mut fund = Fund::new(Some(500), Some(1000));
        fund.spend(250);
        assert_eq!(fund.amount, 250);
    }
}