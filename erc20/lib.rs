#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod erc20 {
    use ink::storage::Mapping;
    use ink_prelude::string::String;

    #[ink(storage)]
    pub struct Erc20 {
        name: String,
        symbol: String,
        decimals: u8,
        total_supply: Balance,
        balances: ink::storage::Mapping<AccountId, Balance>,
        allowances: ink::storage::Mapping<(AccountId, AccountId), Balance>
    }

    #[ink(event)]
    pub struct Transfer {
        from: AccountId,
        to: AccountId,
        value: Balance
    }

    #[ink(event)]
    pub struct Approval {
        owner: AccountId,
        spender: AccountId,
        value: Balance
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        InsufficientBalance,
        InsufficientAllowance
    }

    pub type Result<T> = core::result::Result<T, Error>;

    impl Erc20 {
        #[ink(constructor)]
        pub fn new(name: String, symbol: String, decimals: u8, total_supply: Balance) -> Self {
            let caller = Self::env().caller();
            let mut balances = Mapping::default();
            balances.insert(caller, &total_supply);
            Self::env().emit_event(Transfer {
                from: AccountId::default(),
                to: caller,
                value: total_supply
            });

            return Self {
                name: name,
                symbol: symbol,
                decimals: decimals,
                total_supply: total_supply,
                balances: balances,
                allowances: Mapping::default()
            };
        }

        pub fn name(&self) -> &String {
            return &self.name;
        }

        pub fn symbol(&self) -> &String {
            return &self.symbol;
        }

        pub fn decimals(&self) -> u8 {
            return self.decimals;
        }

        #[ink(message)]
        pub fn total_supply(&self) -> Balance {
            return self.total_supply;
        }

        #[ink(message)]
        pub fn balance_of(&self, account: AccountId) -> Balance {
            match self.balances.get(account) {
                Some(v) => v,
                None => 0
            }
        }

        #[ink(message)]
        pub fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
            self.allowances.get((owner, spender)).unwrap_or_default()
        }

        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, amount: Balance) -> Result<()> {
            let from: AccountId = self.env().caller();
            let from_balance = self.balances.get(from).unwrap_or_default();
            if from_balance < amount {
                return Err(Error::InsufficientBalance);
            }

            self.balances.insert(from, &(from_balance - amount));
            let to_balance = self.balances.get(to).unwrap_or_default();
            self.balances.insert(to, &(to_balance + amount));
            self.env().emit_event(Transfer{
                from: from,
                to: to,
                value: amount
            });
            Ok(())
        }

        #[ink(message)]
        pub fn transfer_from(&mut self, from: AccountId, to: AccountId, amount: Balance) -> Result<()> {
            let caller = self.env().caller();
            let caller_allowance = self.allowance(from, caller);

            if caller_allowance < amount {
                return Err(Error::InsufficientAllowance);
            }

            self.allowances.insert((from, caller), &(caller_allowance - amount));

            let from_balance = self.balance_of(from);
            if from_balance < amount {
                return Err(Error::InsufficientBalance);
            }

            self.balances.insert(from, &(from_balance - amount));
            let to_balance = self.balance_of(to);
            self.balances.insert(to, &(to_balance + amount));

            self.env().emit_event(Transfer{
                from: from,
                to: to,
                value: amount
            });
            Ok(())
        }

        #[ink(message)]
        pub fn approve(&mut self, spender: AccountId, amount: Balance) -> Result<()> {
            let owner = self.env().caller();
            self.allowances.insert((owner, spender), &amount);
            self.env().emit_event(Approval{
                owner: owner,
                spender: spender,
                value: amount
            });

            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::{test, DefaultEnvironment};

        #[ink::test]
        fn constructor_works() {
            let accounts = test::default_accounts::<DefaultEnvironment>();
            let mint_amount = 10_000_000;
            let mut erc20 = Erc20::new(
                String::from("TestToken"), String::from("TT"), 18, mint_amount
            );

            assert_eq!(erc20.name(), "TestToken");
            assert_eq!(erc20.symbol(), "TT");
            assert_eq!(erc20.decimals(), 18);
            assert_eq!(erc20.balance_of(accounts.alice), mint_amount);
            assert_eq!(erc20.total_supply(), mint_amount);
        }

        #[ink::test]
        fn balance_of_works() {
            let accounts = test::default_accounts::<DefaultEnvironment>();
            let mint_amount = 10_000_000;

            let mut erc20 = Erc20::new(
                String::from("TestToken"), String::from("TT"), 18, mint_amount
            );
            let alice_balance = erc20.balance_of(accounts.alice);

            assert_eq!(alice_balance, mint_amount);
        }

        #[ink::test]
        fn allowance_works() {
            let accounts = test::default_accounts::<DefaultEnvironment>();
            let mint_amount = 10_000_000;

            let mut erc20 = Erc20::new(
                String::from("TestToken"), String::from("TT"), 18, mint_amount
            );

            assert_eq!(erc20.allowance(accounts.alice, accounts.bob), 0);
        }

        #[ink::test]
        fn approve_works() {
            let accounts = test::default_accounts::<DefaultEnvironment>();
            let mint_amount = 10_000_000;
            let amount = 10_000;

            let mut erc20 = Erc20::new(
                String::from("TestToken"), String::from("TT"), 18, mint_amount
            );

            erc20.approve(accounts.bob, amount);
            assert_eq!(erc20.allowance(accounts.alice, accounts.bob), amount);
        }

        #[ink::test]
        fn transfer_works() {
            let accounts = test::default_accounts::<DefaultEnvironment>();
            let mint_amount = 10_000_000;
            let amount = 10_000;

            let mut erc20 = Erc20::new(
                String::from("TestToken"), String::from("TT"), 18, mint_amount
            );

            let alice_balance = erc20.balance_of(accounts.alice);
            let bob_balance = erc20.balance_of(accounts.bob);

            erc20.transfer(accounts.bob, amount); 

            assert_eq!(erc20.balance_of(accounts.alice), alice_balance - amount);
            assert_eq!(erc20.balance_of(accounts.bob), bob_balance + amount);
        }

        #[ink::test]
        fn transfer_from_works() {
            let accounts = test::default_accounts::<DefaultEnvironment>();
            let mint_amount = 10_000_000;
            let amount = 10_000;

            let mut erc20 = Erc20::new(
                String::from("TestToken"), String::from("TT"), 18, mint_amount
            );

            let alice_balance = erc20.balance_of(accounts.alice);
            let bob_balance = erc20.balance_of(accounts.bob);

            assert_eq!(erc20.approve(accounts.eve, amount), Ok(()));

            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.eve);
            assert_eq!(erc20.transfer_from(accounts.alice, accounts.bob, amount), Ok(()));

            assert_eq!(erc20.balance_of(accounts.alice), alice_balance - amount);
            assert_eq!(erc20.balance_of(accounts.bob), bob_balance + amount);
        }
    }
}
