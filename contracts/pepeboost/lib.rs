// SPDX-License-Identifier: MIT #![cfg_attr(not(feature = "std"), no_std, no_main)]
#![cfg_attr(not(feature = "std"), no_std, no_main)]
#[pendzl::implementation(PSP22, PSP22Burnable, PSP22Metadata, Ownable)]
#[ink::contract]
pub mod psp22 {
    use ink::{
        prelude::string::String,
        storage::{traits::ManualKey, Mapping},
    };

    use pendzl::contracts::psp22::*;

    #[ink(event)]
    pub struct NftStatsUpdated {
        #[ink(topic)]
        nft_id: u16,
        #[ink(topic)]
        stats: [u8; 6],
    }

    #[ink(storage)]
    #[derive(Default, StorageFieldGetter)]
    pub struct Contract {
        #[storage_field]
        psp22: PSP22Data,
        #[storage_field]
        metadata: PSP22MetadataData,
        #[storage_field]
        ownable: OwnableData,
        nfts_stats: Mapping<u16, [u8; 6], ManualKey<123>>,
        upgrade_cost: Balance,
        games: Mapping<AccountId, bool>,
    }

    impl Contract {
        #[ink(constructor)]
        pub fn new(
            total_supply: Balance,
            name: Option<String>,
            symbol: Option<String>,
            decimal: u8,
        ) -> Self {
            let mut instance = Self::default();
            let caller = instance.env().caller();
            instance._update_owner(&Some(caller));
            instance.metadata.name.set(&name);
            instance.metadata.symbol.set(&symbol);
            instance.metadata.decimals.set(&decimal);

            instance
                ._update(None, Some(&caller), &total_supply)
                .expect("Should mint total_supply");

            instance
        }

        #[ink(message)]
        pub fn get_nft_stats(&self, nft_id: u16) -> Option<[u8; 6]> {
            self.nfts_stats.get(&nft_id)
        }

        #[ink(message)]
        pub fn add_nft_stats(&mut self, nft_id: u16, index: u8) {
            let caller = self.env().caller();
            let decimals = self.metadata.decimals.get().unwrap_or(0);
            let burn_amount = self.upgrade_cost * 10u128.pow(decimals.into());

            // Burn 6 tokens from the caller's balance
            self._burn_from(&caller, &burn_amount)
                .expect("Should burn tokens");

            let mut stats = self.nfts_stats.get(&nft_id).unwrap_or([0; 6]);
            if index < 6 {
                stats[index as usize] += 1;
            }
            self.nfts_stats.insert(&nft_id, &stats);

            Self::env().emit_event(NftStatsUpdated { nft_id, stats });
        }

        #[ink(message, payable)]
        pub fn start_game(&mut self) {
            let caller = self.env().caller();
            let value = self.env().transferred_value();
            assert!(
                value >= 4_000_000_000,
                "Must pay at least 0.004 $AZERO fees to start the game"
            );
            self.games.insert(&caller, &true);
        }

        #[ink(message)]
        pub fn check_game_status(&self, user: AccountId) -> bool {
            self.games.get(&user).unwrap_or(false)
        }

        #[ink(message)]
        pub fn end_game(
            &mut self,
            user: AccountId,
            mint_amount: Balance,
        ) -> Result<(), PSP22Error> {
            self._only_owner()?;

            // Check if the user has an ongoing game
            let has_ongoing_game = self.check_game_status(user);

            // Assert that the user has an ongoing game
            assert!(has_ongoing_game, "User does not have an ongoing game");

            // End the game for the user
            self.games.insert(&user, &false);

            // Mint the specified amount of tokens
            Self::default()
                ._mint_to(&user, &mint_amount)
                .expect("Should mint tokens");

            Ok(())
        }
    }
}
