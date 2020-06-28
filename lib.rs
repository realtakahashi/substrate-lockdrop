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
        fn lock(&mut self, _minutes: u16) -> Result<(), Error> {
            if self.total_supply < self.env().transferred_balance() {
                return Err(Error::NotEnoughBalance);
            }

            self.lock_balance
                .insert(self.env().caller(), self.env().transferred_balance());
            self.lock_time
                .insert(self.env().caller(), self.env().block_timestamp());
            self.balances
                .insert(self.env().caller(), self.env().transferred_balance());
            let t_supply: u128 = *self.total_supply.get();
            self.total_supply
                .set(t_supply - self.env().transferred_balance());
            Ok(())
        }

        fn balance_of_or_zero(&self, owner: &AccountId) -> Balance {
            *self.balances.get(owner).unwrap_or(&0)
        }

        /// unlock function
        #[ink(message)]
        fn unlock(&mut self) -> Result<(), Error> {
            let mut lock_time: u64 = 0;
            match self.lock_time.get(&self.env().caller()).cloned() {
                Some(result) => lock_time = result,
                None => return Err(Error::NotSpendLockTime),
            }
            if self.env().block_timestamp() < lock_time {
                return Err(Error::NotSpendLockTime);
            }
            if let Err(e) = self.send_unlock() {
                return Err(e);
            }
            self.lock_balance.remove(&self.env().caller());
            Ok(())
        }

        fn send_unlock(&self) -> Result<(), Error> {
            let mut lock_b: u128 = 0;
            match self.lock_balance.get(&self.env().caller()).cloned() {
                Some(result) => lock_b = result,
                None => return Err(Error::NoValue),
            }
            self.env().transfer(self.env().caller(), lock_b);
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

        /// We test if the default constructor does its job.
        #[test]
        fn default_works() {
            // Note that even though we defined our `#[ink(constructor)]`
            // above as `&mut self` functions that return nothing we can call
            // them in test code as if they were normal Rust constructors
            // that take no `self` argument but return `Self`.
            let lockdrop = Lockdrop::default();
            assert_eq!(lockdrop.get(), false);
        }

        /// We test a simple use case of our contract.
        #[test]
        fn it_works() {
            let mut lockdrop = Lockdrop::new(false);
            assert_eq!(lockdrop.get(), false);
            lockdrop.flip();
            assert_eq!(lockdrop.get(), true);
        }
    }
}
