//! # Fund Manager
//! 
//! This is a crate I wrote as a library for my FundWarrior project,
//! a simple command line money management tool. I decided to split
//! it into a separate library to make it easier to reuse later, if
//! I or anyone else wished to make a GUI version of FundWarrior for
//! example.

use std::collections::HashMap;
use std::collections::hash_map::IntoIter;
use std::iter::FromIterator;
use std::error::Error;
use std::fmt;
use std::fs;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::path::Path;

/// Manages storage and retrieval of Funds
pub struct FundManager {
    funds: HashMap<String, Fund>,
}

impl FundManager {
    /// Returns a new FundManager based on the contents of the
    /// specified file
    /// 
    /// # Arguments
    /// 
    /// * `fundfile` - the location of the 'fund' file
    /// 
    /// # Errors
    /// 
    /// * When the directories could not be created
    /// * When the file could not be opened
    /// * When the file could not be parsed correctly
    pub fn load(fundfile: &Path) -> Result<FundManager, Box<Error + Send + Sync>> {
        fs::create_dir_all(fundfile.parent().unwrap_or(fundfile))?;
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

    /// Saves FundManager to a file and Returns either the unit type or an Error
    /// 
    /// # Arguments
    /// 
    /// * `fundfile` - the location of the 'fund' file
    /// 
    /// # Errors
    /// 
    /// * When the specified directory and/or parent directories
    /// could not be created
    /// * When the 'fund' file could not be created or opened
    /// * When the 'fund' file could not be written to
    pub fn save(self, fundfile: &Path) -> Result<(), Box<Error + Send + Sync>> {
        fs::create_dir_all(fundfile.parent().unwrap_or(fundfile))?;
        let file = OpenOptions::new().write(true).create(true).open(fundfile)?;
        let mut buf_writer = BufWriter::new(file);
        for fund in self.funds {
            let string = format!("{}:{}:{}\n", fund.0, fund.1.amount, fund.1.goal);
            buf_writer.write_all(string.as_bytes())?;
        }
        Ok(())
    }

    /// Returns a Result containing either an Error message or a mutable reference
    /// to the fund with the specified name
    /// 
    /// # Arguments
    /// 
    /// * `name` - a string slice representing the name of the fund you want
    /// 
    /// # Errors
    /// 
    /// * When the fund cannot be found
    pub fn get_fund_by_name(&mut self, name: &str) -> Result<&mut Fund, &'static str> {
        match self.funds.get_mut(name) {
            Some(fund) => Ok(fund),
            None => Err("cannot find the fund"),
        }
    }

    /// Prints information about the fund with the given name to stdout, or returns an
    /// Error if the fund could not be found
    /// 
    /// # Arguments
    /// 
    /// * `name` - a string slice representing the name of the fund you want to print
    /// 
    /// # Errors
    /// 
    /// * When the fund cannot be found
    pub fn print_fund(&mut self, name: &str) -> Result<(), Box<Error + Send + Sync>> {
        let fund = self.get_fund_by_name(name)?;
        let mut name = String::from(name);
        name.push(':');
        println!("{:>10} {}", name, fund);
        Ok(())
    }

    /// Prints information about all funds the FundManager is currently
    /// storing
    pub fn print_all(self) {
        for fund in self {
            let mut name = fund.0.to_owned();
            name.push(':');
            println!("{:>10} {}", name, fund.1)
        }
    }

    /// Adds a new Fund to the FundManager
    /// 
    /// # Arguments
    /// 
    /// * `name` - the name of the new fund
    /// * `amount` - either `Some(x)`, where x is the starting amount of the new fund
    ///  or `None` in which case the starting amount is 0
    /// * `goal` - either `Some(x)`, where x is the goal for this fund, or `None` in which case the goal is 0
    /// 
    /// # Errors
    /// 
    /// * When attempting to add a fund with a name that is already in use
    pub fn add_fund(
        &mut self,
        name: &str,
        fund: Fund
    ) -> Result<(), Box<Error + Send + Sync>> {
        if self.funds.contains_key(name) {
            return Err(From::from(format!(
                "fund '{}' already exists. Please choose a different name",
                name
            )));
        }
        self.funds
            .insert(String::from(name), fund);
        Ok(())
    }
}

impl IntoIterator for FundManager {
    type Item = (String, Fund);
    type IntoIter = IntoIter<String, Fund>;

    fn into_iter(self) -> Self::IntoIter {
        self.funds.into_iter()
    }
}

impl FromIterator<(String, Fund)> for FundManager {

    fn from_iter<I: IntoIterator<Item=(String, Fund)>>(iter: I) -> Self {
        let mut funds: HashMap<String, Fund> = HashMap::new();
        for fund in iter {
            funds.insert(fund.0, fund.1);
        }
        FundManager{ funds }
    }
}

/// Stores and manipulates a running balance and goal to shoot for
#[derive(Debug, Default, PartialEq)]
pub struct Fund {
    pub amount: i32,
    pub goal: i32,
}

impl Fund {
    /// Returns a new Fund with default amounts of 0 for amount and goal
    pub fn new() -> Fund {
        Fund {
            amount: 0,
            goal: 0,
        }
    }

    /// Sets `self`'s amount and returns a reference to itself.
    /// Intended for use as part of a builder pattern.
    /// 
    /// # Example
    /// ```
    /// use libfund::Fund;
    /// 
    /// let fund = Fund::new().with_amount(100).build();
    /// assert_eq!(fund.amount, 100);
    /// assert_eq!(fund.goal, 0);
    /// ```
    pub fn with_amount(&mut self, amount: i32) -> &mut Self {
        self.amount = amount;
        self
    }

    /// Sets `self`'s goal and returns a reference to itself.
    /// Intended for use as part of a builder pattern.assert_eq!
    /// # Example
    /// ```
    /// use libfund::Fund;
    /// 
    /// let fund = Fund::new().with_goal(500).build();
    /// assert_eq!(fund.amount, 0);
    /// assert_eq!(fund.goal, 500);
    pub fn with_goal(&mut self, goal: i32) -> &mut Self {
        self.goal = goal;
        self
    }

    /// Returns a new fund based on itself and consumes its reference. 
    /// Intended as the last step of a builder pattern.
    /// 
    /// # Example
    /// ```
    /// use libfund::Fund;
    /// 
    /// let fund = Fund::new().with_amount(100).with_goal(500).build();
    /// assert_eq!(fund.amount, 100 );
    /// assert_eq!(fund.goal, 500);
    /// ```
    pub fn build(&self) -> Fund {
        Fund{
            amount: self.amount,
            goal: self.goal,
        }
    }

    /// Decreases the amount stored in the Fund
    /// 
    /// # Arguments
    /// 
    /// * `amount` - The amount of money to subtract from the fund
    pub fn spend(&mut self, amount: i32) {
        self.amount -= amount;
    }

    /// Increases the amount stored in the Fund
    /// 
    /// # Arguments
    /// 
    /// * `amount` - The amount of money to add to the fund
    pub fn deposit(&mut self, amount: i32) {
        self.amount += amount;
    }
}

impl fmt::Display for Fund {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:^8} / {:<8} -- {} away from goal",
            display_dollars(self.amount),
            display_dollars(self.goal),    //use std::path::PathBuf;

            display_dollars(self.goal - self.amount)
        )
    }
}

fn display_dollars(amount: i32) -> String {
        let mut amount = amount.to_string();
        while amount.len() < 3 {
            amount.insert(0, '0');
        }
        let (dollars, cents) = amount.split_at(amount.len() - 2);
        format!("${}.{}", dollars, cents)
    }

#[cfg(test)]
mod tests {
    use super::{FundManager, Fund, display_dollars};
    use std::collections::HashMap;
    use std::env;

    #[test]
    fn create_fund() {
        let fund = Fund::new();
        assert_eq!(fund.amount, 0);
        assert_eq!(fund.goal, 0);
        let fund_with_args = Fund::new().with_amount(500).with_goal(1000).build();
        assert_eq!(fund_with_args.amount, 500);
        assert_eq!(fund_with_args.goal, 1000);
    }

    #[test]
    fn fund_deposit() {
        let mut fund = Fund::new().with_amount(500).with_goal(1000).build();
        fund.deposit(500);
        assert_eq!(fund.amount, 1000);
    }
    #[test]
    fn fund_spend() {
        let mut fund = Fund::new().with_amount(500).with_goal(1000).build();
        fund.spend(250);
        assert_eq!(fund.amount, 250);
    }

    #[test]
    fn dollar_display() {
        let amount = 100;
        assert_eq!(display_dollars(amount), "$1.00");
    }

    #[test]
    fn display() {
        let fund = Fund::new().with_amount(500).with_goal(1000).build();
        assert_eq!(format!("{}", fund), format!(
            "{:^8} / {:<8} -- {} away from goal",
            display_dollars(fund.amount),
            display_dollars(fund.goal),
            display_dollars(fund.goal - fund.amount)
        ));
    }

    #[test]
    fn adding_funds() {
        let mut funds = FundManager{ funds: HashMap::new() };
        let result = funds.add_fund("Test", Fund::new().with_amount(100).with_goal(500).build());
        assert!(result.is_ok());
        assert_eq!(funds.funds.len(), 1);
        assert!(funds.funds.contains_key("Test"));
        assert!(funds.add_fund("Test", Fund::new()).is_err());
    }
    #[test]
    fn getting_funds() {
        let mut funds = FundManager{ funds: HashMap::new() };
        funds.add_fund("Test", Fund::new().with_amount(100).with_goal(500).build()).unwrap();
        assert!(funds.get_fund_by_name("Test").is_ok());
        assert!(funds.get_fund_by_name("NotHere").is_err());
    }

    #[test]
    fn load_and_save() {
        let mut test_data = env::current_dir().unwrap();
        test_data.push(r"test_data/fund");
        let result = FundManager::load(&test_data);
        assert!(result.is_ok());
        let funds = result.unwrap();
        let result = funds.save(&test_data);
        assert!(result.is_ok());
    }
}
