// SPDX-License-Identifier: MIT
#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[pendzl::implementation(PSP34, PSP34Metadata, PSP34Mintable, Ownable)]
#[ink::contract]
pub mod my_psp34_metadata {
    use ink::prelude::string::*;
    use pendzl::contracts::psp34::*;

    #[derive(Default, StorageFieldGetter)]
    #[ink(storage)]
    pub struct Contract {
        #[storage_field]
        psp34: PSP34Data,
        #[storage_field]
        metadata: PSP34MetadataData,
        #[storage_field]
        ownable: OwnableData,
        next_id: u64,
        max_supply: u64,
        price_per_mint: Balance,
    }

    impl Contract {
        #[ink(constructor)]
        pub fn new(
            name: String,
            symbol: String,
            base_uri: String,
            max_supply: u64,
            price_per_mint: Balance,
        ) -> Self {
            let mut instance = Self::default();
            let caller = instance.env().caller();
            let name_key = String::from("name");
            let symbol_key = String::from("symbol");
            let base_uri_key = String::from("baseURI");
            instance._update_owner(&Some(caller));
            instance._set_attribute(&Id::U64(0), &name_key, &name);
            instance._set_attribute(&Id::U64(0), &symbol_key, &symbol);
            instance._set_attribute(&Id::U64(0), &base_uri_key, &base_uri);
            // would be nice:
            // instance.metadata.name.set(&name);
            // instance.metadata.symbol.set(&symbol);
            // instance.metadata.decimals.set(&decimal);
            instance.max_supply = max_supply;
            instance.price_per_mint = price_per_mint;
            instance.next_id = 0;
            instance
        }

        #[ink(message, payable)]
        pub fn mint(&mut self, mint_amount: u64) -> Result<(), PSP34Error> {
            self.check_amount(mint_amount)?;
            self.check_value(Self::env().transferred_value(), mint_amount)?;
            let next_to_mint = self.next_id;
            let mint_offset = next_to_mint + mint_amount;
            for _mint_id in next_to_mint..mint_offset {
                self.mint_token()?;
            }
            match mint_amount {
                10 => {
                    self.mint_token()?;
                }
                20 => {
                    for _i in 0..3 {
                        self.mint_token()?;
                    }
                }
                _ => (),
            }
            Ok(())
        }

        fn mint_token(&mut self) -> Result<(), PSP34Error> {
            self._mint_to(&Self::env().caller(), &Id::U64(self.next_id))?;
            self.next_id = self.next_id.checked_add(1).unwrap();
            Ok(())
        }

        fn check_amount(&self, amount: u64) -> Result<(), PSP34Error> {
            if amount == 0 {
                return Err(PSP34Error::Custom(String::from(
                    "Invalid mint amount",
                )));
            }
            Ok(())
        }

        fn check_value(
            &self,
            value: Balance,
            amount: u64,
        ) -> Result<(), PSP34Error> {
            let required_value = self.price_per_mint * amount as Balance;
            if value < required_value {
                return Err(PSP34Error::Custom(String::from(
                    "Insufficient payment",
                )));
            }
            Ok(())
        }

        #[ink(message)]
        pub fn get_user_tokens(&self, user: AccountId) -> Vec<Id> {
            let mut user_tokens = Vec::new();
            let max_token_id = self.next_id;
            for token_id in 1..=max_token_id {
                let owner = self.psp34.owner_of(&Id::U64(token_id));
                if let Some(owner) = owner {
                    if owner == user {
                        user_tokens.push(Id::U64(token_id));
                    }
                }
            }
            user_tokens
        }
    }
}
