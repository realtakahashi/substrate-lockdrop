#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract(version = "0.1.0")]
mod lockdrop {
    //    use ink_core::memory::string::String;
    use ink_core::storage;
    use scale::{Decode, Encode};

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    struct Lockdrop {
        /// Stores a single `bool` value on the storage.
        total_supply: storage::Value<Balance>,
        balances: storage::HashMap<AccountId, Balance>,
        lock_balance: storage::HashMap<AccountId, Balance>,
        lock_time: storage::HashMap<AccountId, Timestamp>,
        //name: String,
    }

    // #[ink(storage)]
    // struct LockData {
    //     balance: Balance,
    //     time: Timestamp,
    // }

    #[ink(event)]
    struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        #[ink(topic)]
        value: Balance,
    }

    #[derive(Encode, Decode, Debug, PartialEq, Eq, Copy, Clone)]
    #[cfg_attr(feature = "ink-generate-abi", derive(type_metadata::Metadata))]
    pub enum Error {
        NotEnoughBalance,
        NotSpendLockTime,
        NoValue,
        SendFailed,
    }

    #[ink(event)]
    struct Lock {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        value: Balance,
    }

    #[ink(event)]
    struct UnLock {
        #[ink(topic)]
        to: Option<AccountId>,
        #[ink(topic)]
        value: Balance,
    }

    impl Lockdrop {
        /// constructor
        #[ink(constructor)]
        fn new(&mut self, initial_supply: Balance) {
            let caller = self.env().caller();
            self.total_supply.set(initial_supply);
            self.balances.insert(caller, initial_supply);
            //self.name = token_name;
            self.env().emit_event(Transfer {
                from: None,
                to: Some(caller),
                value: initial_supply,
            });
        }

        /// get total_supply of token
        #[ink(message)]
        fn total_supply(&self) -> Balance {
            *self.total_supply
        }

        /// get my token balance
        #[ink(message)]
        fn balance_of_token(&self, owner: AccountId) -> Balance {
            self.balance_of_or_zero(&owner)
        }

        /// get my lock balance
        #[ink(message)]
        fn balance_of_lock(&self, owner: AccountId) -> Balance {
            let balance = *self.lock_balance.get(&owner).unwrap_or(&0);
            balance
        }

        /// lock function
        #[ink(message)]
        fn lock(&mut self, milliseconds: u64) -> Result<(), Error> {
            if self.total_supply < self.env().transferred_balance() {
                return Err(Error::NotEnoughBalance);
            }

            let to_time = self.env().block_timestamp() + milliseconds;

            self.lock_balance
                .insert(self.env().caller(), self.env().transferred_balance());
            self.lock_time.insert(self.env().caller(), to_time);
            self.balances
                .insert(self.env().caller(), self.env().transferred_balance());
            self.total_supply
                .set(*self.total_supply.get() - self.env().transferred_balance());
            Ok(())
        }

        fn balance_of_or_zero(&self, owner: &AccountId) -> Balance {
            *self.balances.get(owner).unwrap_or(&0)
        }

        fn get_lock_time(&self, owner: &AccountId) -> Timestamp {
            *self.lock_time.get(&owner).unwrap_or(&0)
        }

        #[ink(message)]
        fn pub_get_lock_time(&self, owner: AccountId) -> Timestamp {
            *self.lock_time.get(&owner).unwrap_or(&0)
        }

        #[ink(message)]
        fn pub_get_block_time(&self) -> Timestamp {
            self.env().block_timestamp()
        }

        /// unlock function
        #[ink(message)]
        fn unlock(&mut self) -> Result<(), Error> {
            if self.env().block_timestamp() < self.get_lock_time(&self.env().caller()) {
                return Err(Error::NotSpendLockTime);
            }
            if let Err(e) = self.send_unlock() {
                return Err(e);
            }
            self.lock_balance.remove(&self.env().caller());
            Ok(())
        }

        fn send_unlock(&self) -> Result<(), Error> {
            //let lock_b: Balance = self.balance_of_lock(self.env().caller());
            //let lock_b: Balance;
            // let lock_b: u128;
            // match self.lock_balance.get(&self.env().caller()).cloned() {
            //     Some(result) => lock_b = result,
            //     None => return Err(Error::NoValue),
            // }
            if let Err(_) = self.env().transfer(
                self.env().caller(),
                self.balance_of_lock(self.env().caller()),
            ) {
                return Err(Error::SendFailed);
            }
            Ok(())
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;
        use ink_core::env;

        /// We test a simple use case of our contract.
        #[test]
        fn lock() {
            let accounts =
                env::test::default_accounts::<env::DefaultEnvTypes>().expect("Cannot get accounts");
            let mut lockdrop = Lockdrop::new(9999999999);
            assert_eq!(lockdrop.lock(30), Ok(()));
        }
    }
}
