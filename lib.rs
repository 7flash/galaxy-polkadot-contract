#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod galaxy_contract {
    use ink::prelude::vec::Vec;
    use ink::storage::Mapping;
    use ink::prelude::string::String;

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
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
            let layers = self.user_layers.get(&user).unwrap_or_default().clone();

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
                    if let Some(link) = self.layer_links.get(&(user, layer_name.clone())) {
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

            assert_eq!(contract.resolve_link(caller, "Layer1".to_string()), Err(Error::LayerNotFound));

            contract.create_layer("Layer1".to_string(), "ipfs://link1".to_string()).unwrap();

            let resolve_result = contract.resolve_link(caller, "Layer1".to_string());
            assert_eq!(resolve_result, Ok("ipfs://link1".to_string()));
        }
    }
}
