![fundwarrior logo](./fundwarrior.png)
# FundWarrior

A command line application for money managment inspired
by TaskWarrior. The goal is for any user to be able to
split their accounts into virtual "funds".

For example, say you have $1000 in your account and you
want to save certain amounts of money for certain categories
of spending. Say you want to set aside $100 for groceries and
in the future you want to try and maintain $150 in that fund
whenever possible. You can set up a fund called "grocery" that has
$100 in it with a "goal" of $150 with the following command.

```
fund new grocery 100.00 150.00
```

Currently the names are case-sensitive and cannot
contain any spaces.

You can then view this fund at any time with the command.

```
fund list grocery
```

Now say you want to set aside $500 for car repairs

```
fund new car 500.00 500.00
```

If you want to view this fund

```
fund list car
```

Or if you want to view all your funds

```
fund list
```

Or just

```
fund
```

If you buy $50 worth of groceries, run the command

```
fund spend grocery 50.00
```

Payday comes around and you wish to deposit $50 into your car and grocery funds

```
fund deposit car 50.00
fund deposit grocery 50.00
```

This is a WIP and currently in a rough state. Code is messy at points and presentation is potentially lacking.

## TODO

- Add documentation comments
- Add more configuration options that can be parsed from the config file
- Refactor code to improve clarity and argument flexibility
- Add a command to transfer money between accounts
- Add spending history functionality
- Add ways to view statistics, possibly in graph form