//! # Fund Manager
//!
//! This is a crate I wrote as a library for my FundWarrior project,
//! a simple command line money management tool. I decided to split
//! it into a separate library to make it easier to reuse later, if
//! I or anyone else wished to make a GUI version of FundWarrior for
//! example.
//! 
//! ## Warning
//! 
//! The `FundManager` struct implements `Extend`, but it has a caveat.
//! Any `Fund`s in the supplied iterator that have the same name as any
//! existing `Fund` will be ignored.

use std::cmp::Ordering;
use std::collections::hash_map::{Iter, IterMut};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fs;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::iter::FromIterator;
use std::path::Path;

/// The error returned when a fund could not be found
#[derive(Debug)]
pub struct FundNotFoundError {
    name: String,
}

impl fmt::Display for FundNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "fund '{}' not found", self.name)
    }
}

impl Error for FundNotFoundError {
    fn description(&self) -> &str {
        "fund was not found"
    }
}

/// The error returned when attempting to create or rename a
/// fund to a name that already exists
#[derive(Debug)]
pub struct DuplicateFundError {
    name: String,
}

impl fmt::Display for DuplicateFundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "fund '{}' already exists. Please choose a different name",
            self.name
        )
    }
}

impl Error for DuplicateFundError {
    fn description(&self) -> &str {
        "funds must have unique names"
    }
}

/// A wrapper around FundNotFoundError, DuplicateFundError,
/// and std::io::Error. Useful for binary crates dealing with
/// `FundManager`s, as they may need to deal with any combination
/// of these errors at once.
///
#[derive(Debug)]
pub enum FundManagerError {
    FundNotFound(FundNotFoundError),
    DuplicateFund(DuplicateFundError),
    Io(std::io::Error),
}

impl fmt::Display for FundManagerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            FundManagerError::FundNotFound(ref e) => e.fmt(f),
            FundManagerError::DuplicateFund(ref e) => e.fmt(f),
            FundManagerError::Io(ref e) => e.fmt(f),
        }
    }
}

impl Error for FundManagerError {
    fn description(&self) -> &str {
        match *self {
            FundManagerError::FundNotFound(ref e) => e.description(),
            FundManagerError::DuplicateFund(ref e) => e.description(),
            FundManagerError::Io(ref e) => e.description(),
        }
    }
}

impl From<FundNotFoundError> for FundManagerError {
    fn from(err: FundNotFoundError) -> FundManagerError {
        FundManagerError::FundNotFound(err)
    }
}

impl From<DuplicateFundError> for FundManagerError {
    fn from(err: DuplicateFundError) -> FundManagerError {
        FundManagerError::DuplicateFund(err)
    }
}

impl From<std::io::Error> for FundManagerError {
    fn from(err: std::io::Error) -> FundManagerError {
        FundManagerError::Io(err)
    }
}

/// Manages storage and retrieval of Funds
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct FundManager {
    funds: HashMap<String, Fund>,
}

impl FundManager {
    pub fn new() -> FundManager {
        FundManager {
            funds: HashMap::new(),
        }
    }
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
    pub fn load(fundfile: &Path) -> Result<FundManager, std::io::Error> {
        fs::create_dir_all(fundfile.parent().unwrap_or(fundfile))?;
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&fundfile)?;
        let mut funds: Vec<(String, Fund)> = Vec::new();
        let buf_reader = BufReader::new(file);

        for line in buf_reader.lines() {
            let line = line?;
            let fund_info: Vec<&str> = line.split_terminator(':').collect();
            if fund_info.len() < 3 {
                return Err(From::from(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("{:?} is invalid", fundfile),
                )));
            }
            let name = match fund_info[0].parse() {
                Ok(name) => name,
                Err(e) => {
                    return Err(From::from(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("while parsing {:?}: {}", fundfile, e),
                    )))
                }
            };
            let amount: i32 = match fund_info[1].parse() {
                Ok(amount) => amount,
                Err(e) => {
                    return Err(From::from(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("while parsing {:?}: {}", fundfile, e),
                    )))
                }
            };
            let goal: i32 = match fund_info[2].parse() {
                Ok(goal) => goal,
                Err(e) => {
                    return Err(From::from(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("while parsing {:?}: {}", fundfile, e),
                    )))
                }
            };
            funds.push((
                name,
                Fund::new().with_amount(amount).with_goal(goal).build(),
            ));
        }

        Ok(funds.into_iter().collect())
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
    pub fn save(&self, fundfile: &Path) -> Result<(), std::io::Error> {
        fs::create_dir_all(fundfile.parent().unwrap_or(fundfile))?;
        let file = OpenOptions::new().write(true).create(true).open(fundfile)?;
        let mut buf_writer = BufWriter::new(file);
        for fund in self {
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
    #[deprecated(
        since = "0.8.0",
        note = "please use `fund` or `fund_mut` instead"
    )]
    pub fn get_fund_by_name(&mut self, name: &str) -> Result<&mut Fund, &'static str> {
        match self.funds.get_mut(name) {
            Some(fund) => Ok(fund),
            None => Err("cannot find the fund"),
        }
    }

    /// Takes the name of a `Fund` and returns a reference to it, or an
    /// Error if the `Fund` does not exist
    ///
    /// # Arguments
    ///
    /// * `name` - a string slice containing the name of the `Fund` you want
    ///
    /// # Errors
    ///
    /// * When the `Fund` cannot be found
    ///
    /// # Example
    /// ```
    /// use libfund::{Fund, FundManager};
    ///
    /// let mut funds = FundManager::new();
    /// funds.add_fund("test", Fund::new().with_amount(100).with_goal(500).build());
    /// let fund = funds.fund("test").unwrap();
    /// assert_eq!(fund.amount, 100);
    /// assert_eq!(fund.goal, 500);
    /// ```
    pub fn fund(&self, name: &str) -> Result<&Fund, FundNotFoundError> {
        match self.funds.get(name) {
            Some(fund) => Ok(fund),
            None => Err(FundNotFoundError {
                name: String::from(name),
            }),
        }
    }

    /// Takes the name of a `Fund` and returns a mutable reference to it, or
    /// an Error if the `Fund` does not exist
    ///
    /// # Arguments
    ///
    /// * `name` - a string slice containing the name of the `Fund` you want
    ///
    /// # Errors
    ///
    /// * When the `Fund` cannot be found
    ///
    /// # Example
    /// ```
    /// use libfund::{Fund, FundManager};
    ///
    /// let mut funds = FundManager::new();
    /// funds.add_fund("test", Fund::new().with_amount(100).with_goal(500).build());
    /// let mut fund = funds.fund_mut("test").unwrap();
    /// assert_eq!(fund.amount, 100);
    /// assert_eq!(fund.goal, 500);
    /// fund.amount = 200;
    /// assert_eq!(fund.amount, 200);
    /// ```
    pub fn fund_mut(&mut self, name: &str) -> Result<&mut Fund, FundNotFoundError> {
        match self.funds.get_mut(name) {
            Some(fund) => Ok(fund),
            None => Err(FundNotFoundError {
                name: String::from(name),
            }),
        }
    }

    #[deprecated(
        since = "0.8.0",
        note = "Slated for removal in 1.0.0, please use the getter functions to get the values 
        you want and the `Display` trait on `Fund` and the `display_dollars` function to get the 
        information you want."
    )]
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
    pub fn print_fund(&mut self, name: &str) -> Result<(), FundNotFoundError> {
        let fund = self.fund(name)?;
        let mut name = String::from(name);
        name.push(':');
        println!("{:>10} {}", name, fund);
        Ok(())
    }

    #[deprecated(
        since = "0.8.0",
        note = "Slated for removal in 1.0.0, please use the `into_iter` method on `FundManger`,
        the `Display` trait on `Fund`, and the `display_dollars` helper function to get the
        information you want."
    )]
    /// Prints information about all funds the FundManager is currently
    /// storing
    pub fn print_all(&self) {
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
    pub fn add_fund(&mut self, name: &str, fund: Fund) -> Result<(), DuplicateFundError> {
        if self.funds.contains_key(name) {
            return Err(DuplicateFundError {
                name: String::from(name),
            });
        }
        self.funds.insert(String::from(name), fund);
        Ok(())
    }

    /// Renames a fund from old_name to new_name. Returns an Error if either the new
    /// name is already in the FundManager or the old name wasn't found.
    ///
    /// # Examples
    /// ```
    /// use libfund::{Fund, FundManager};
    ///
    /// let mut funds = FundManager::new();
    /// funds.add_fund("test", Fund::new().with_amount(100).with_goal(200).build());
    /// funds.rename("test", "success");
    /// assert!(funds.fund("test").is_err());
    /// assert!(funds.fund("success").is_ok());
    /// ```
    pub fn rename(&mut self, old_name: &str, new_name: &str) -> Result<(), FundManagerError> {
        match self.funds.remove(old_name) {
            Some(fund) => self.add_fund(new_name, fund)?,
            None => {
                return Err(From::from(FundNotFoundError {
                    name: String::from(old_name),
                }))
            }
        };
        Ok(())
    }
}

impl<'a> IntoIterator for &'a FundManager {
    type Item = (&'a String, &'a Fund);
    type IntoIter = Iter<'a, String, Fund>;

    fn into_iter(self) -> Self::IntoIter {
        self.funds.iter()
    }
}

impl<'a> IntoIterator for &'a mut FundManager {
    type Item = (&'a String, &'a mut Fund);
    type IntoIter = IterMut<'a, String, Fund>;

    fn into_iter(self) -> Self::IntoIter {
        self.funds.iter_mut()
    }
}

impl Extend<(String, Fund)> for FundManager {
    fn extend<I: IntoIterator<Item = (String, Fund)>>(&mut self, iter: I) {
        //! Extends a collection with the contents of an iterator. 
        //! 
        //! Warning!: Does not add funds that have the same name as previously existing funds.
        for fund in iter {
            if !self.funds.contains_key(&fund.0) {
                self.add_fund(&fund.0, fund.1).unwrap();
            }
        }
    }
}

impl<'a> Extend<(&'a String, &'a Fund)> for FundManager {
    fn extend<I: IntoIterator<Item = (&'a String, &'a Fund)>>(&mut self, iter: I) {
        //! Extends a collection with the contents of an iterator.
        //! 
        //! Warning: Does not add funds that have the same name as previously existing funds.
        for fund in iter {
            if !self.funds.contains_key(fund.0) {
                self.add_fund(&fund.0, *fund.1).unwrap();
            }
        }
    }
}

impl FromIterator<(String, Fund)> for FundManager {
    fn from_iter<I: IntoIterator<Item = (String, Fund)>>(iter: I) -> Self {
        let mut funds = HashMap::new();
        for fund in iter {
            funds.insert(fund.0, fund.1);
        }
        FundManager { funds }
    }
}

/// Stores and manipulates a running balance and goal to shoot for
#[derive(Debug, Default, PartialEq, Eq, Hash, Copy, Clone)]
pub struct Fund {
    pub amount: i32,
    pub goal: i32,
}

impl PartialOrd for Fund {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.amount.cmp(&other.amount))
    }
}

impl Ord for Fund {
    fn cmp(&self, other: &Self) -> Ordering {
        self.amount.cmp(&other.amount)
    }
}

impl Fund {
    /// Returns a new Fund with default amounts of 0 for amount and goal
    pub fn new() -> Fund {
        Fund { amount: 0, goal: 0 }
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
        Fund {
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
            display_dollars(self.goal), //use std::path::PathBuf;
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
    use super::{display_dollars, Fund, FundManager};
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
        assert_eq!(
            format!("{}", fund),
            format!(
                "{:^8} / {:<8} -- {} away from goal",
                display_dollars(fund.amount),
                display_dollars(fund.goal),
                display_dollars(fund.goal - fund.amount)
            )
        );
    }

    #[test]
    fn adding_funds() {
        let mut funds = FundManager {
            funds: HashMap::new(),
        };
        let result = funds.add_fund("Test", Fund::new().with_amount(100).with_goal(500).build());
        assert!(result.is_ok());
        assert_eq!(funds.funds.len(), 1);
        assert!(funds.funds.contains_key("Test"));
        assert!(funds.add_fund("Test", Fund::new()).is_err());
    }
    #[test]
    fn getting_funds() {
        let mut funds = FundManager {
            funds: HashMap::new(),
        };
        funds
            .add_fund("Test", Fund::new().with_amount(100).with_goal(500).build())
            .unwrap();
        assert!(funds.fund("Test").is_ok());
        assert!(funds.fund("NotHere").is_err());
    }

    #[test]
    fn get_mutable() {
        let mut funds = FundManager {
            funds: HashMap::new(),
        };
        funds
            .add_fund("Test", Fund::new().with_amount(100).with_goal(500).build())
            .unwrap();
        assert!(funds.fund("Test").is_ok());
        assert!(funds.fund("NotHere").is_err());
        funds.fund_mut("Test").unwrap().amount = 200;
        assert_eq!(funds.fund("Test").unwrap().amount, 200);
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

    #[test]
    fn renames_fund() {
        let mut funds = FundManager::new();
        funds
            .add_fund("test", Fund::new().with_amount(100).with_goal(200).build())
            .unwrap();
        funds.rename("test", "success").unwrap();
        assert!(funds.fund("test").is_err());
        assert!(funds.fund("success").is_ok());
        assert!(funds.rename("test", "success").is_err());
        funds
            .add_fund("test", Fund::new().with_amount(100).with_goal(200).build())
            .unwrap();
        assert!(funds.rename("success", "test").is_err());
    }
}
