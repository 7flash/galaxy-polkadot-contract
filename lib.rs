#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod galaxy_contract {
    use ink_prelude::vec::Vec;
    use ink_storage::{traits::SpreadAllocate, Mapping};

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    pub enum Error {
        LayerAlreadyExists,
        LayerNotFound,
    }

    type Result<T> = core::result::Result<T, Error>;

    #[ink(event)]
    pub struct LayerCreated {
        #[ink(topic)]
        user: AccountId,
        layer_name: String,
        ipfs_link: String,
    }

    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct GalaxyContract {
        user_layers: Mapping<AccountId, Vec<String>>,
        layer_links: Mapping<(AccountId, String), String>,
    }

    impl GalaxyContract {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                user_layers: Default::default(),
                layer_links: Default::default(),
            }
        }

        #[ink(message)]
        pub fn create_layer(&mut self, layer_name: String, ipfs_link: String) -> Result<()> {
            let user = self.env().caller();
            
            let layers = self.user_layers.get(&user).unwrap_or(Vec::new()).to_vec();

            if layers.contains(&layer_name) {
                return Err(Error::LayerAlreadyExists);
            }

            let mut new_layers = layers;
            new_layers.push(layer_name.clone());
            self.user_layers.insert(user, &new_layers);
            self.layer_links.insert((user, layer_name.clone()), &ipfs_link);

            self.env().emit_event(LayerCreated { user, layer_name, ipfs_link });

            Ok(())
        }

        #[ink(message)]
        pub fn resolve_link(&self, user: AccountId, layer_name: String) -> Result<String> {
            if let Some(layers) = self.user_layers.get(&user) {
                if layers.contains(&layer_name) {
                    if let Some(link) = self.layer_links.get(&(user, layer_name)) {
                        return Ok(link.clone());
                    }
                }
            }
            Err(Error::LayerNotFound)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink_lang as ink;
        use ink_env::{test, DefaultEnvironment};

        #[ink::test]
        fn test_create_layer() {
            let mut contract = GalaxyContract::new();
            
            assert_eq!(contract.create_layer("Layer1".to_string(), "ipfs://link1".to_string()), Ok(()));
            assert_eq!(contract.create_layer("Layer1".to_string(), "ipfs://link2".to_string()), Err(Error::LayerAlreadyExists));
        }

        #[ink::test]
        fn test_resolve_link() {
            let mut contract = GalaxyContract::new();

            let caller = AccountId::from([0x1; 32]);
            test::set_caller::<DefaultEnvironment>(caller);

            assert_eq!(contract.resolve_link(caller, "Layer1".to_string()), Err(Error::LayerNotFound));

            let creation_result = contract.create_layer("Layer1".to_string(), "ipfs://link1".to_string());
            assert_eq!(creation_result, Ok(()));

            let user_layers = contract.user_layers.get(&caller);
            ink_env::debug_println!("User layers: {:?}", user_layers);

            let link = contract.layer_links.get(&(caller.clone(), "Layer1".to_string()));
            ink_env::debug_println!("Layer1 link: {:?}", link);

            let resolve_result = contract.resolve_link(caller.clone(), "Layer1".to_string());
            assert_eq!(resolve_result, Ok("ipfs://link1".to_string()));
        }
    }
}
