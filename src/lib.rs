use std::env;
use std::fs;
use std::io;
use std::io::prelude::*;

pub struct Config {
    pub configfile: String,
    pub fundfile: String,
}

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

    pub fn get_by_name(funds: Vec<Fund>, name: String) -> Result<Fund, &'static str> {
        for fund in funds {
            if fund.name == name {
                return Ok(fund);
            }
        }
        Err("can't find a fund with that name")
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
        let fund_with_args = Fund::new(String::from("Test"), Some(50), Some(100)).unwrap();
        assert_eq!(fund_with_args.name, "Test");
        assert_eq!(fund_with_args.amount, 50);
        assert_eq!(fund_with_args.goal, 100);
    }

    #[test]
    fn fund_err_on_blank_name() {
        let fund = Fund::new(String::from(" "), None, None);
        assert!(fund.is_err());
    }

    #[test]
    fn fund_deposit() {
        let mut fund = Fund::new(String::from("Test"), Some(50), Some(100)).unwrap();
        fund.deposit(50);
        assert_eq!(fund.amount, 100);
    }
    #[test]
    fn fund_spend() {
        let mut fund = Fund::new(String::from("Test"), Some(50), Some(100)).unwrap();
        fund.spend(25);
        assert_eq!(fund.amount, 25);
    }
}