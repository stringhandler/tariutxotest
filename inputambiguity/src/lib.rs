use curve25519_dalek::ristretto::CompressedRistretto;
use curve25519_dalek::RistrettoPoint;
use merlin::Transcript;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tari_template_lib::prelude::*;
use triptych::InputSet;
use triptych::Parameters;
use triptych::Proof as TriptychProof;
use triptych::Statement;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ChainRules {
    resource_address: ResourceAddress,
    linking_tags_address: ResourceAddress,
}

#[template]
mod echeck_decoys_template {
    use super::*;

    pub struct ECheckWithDecoy {
        // copied here but should be moved to resource
        chain_rules: ChainRules,
        // num_decoys_required: u32,
        // resource_address: ResourceAddress,
        owner: RistrettoPublicKeyBytes,
        // is_spent: bool,
    }

    impl ECheckWithDecoy {
        pub fn mint_initial_set(
            initial_supply_per_decoy: u32,
            token_symbol: String,
            owner1: RistrettoPublicKeyBytes,
            owner2: RistrettoPublicKeyBytes,
            owner3: RistrettoPublicKeyBytes,
            owner4: RistrettoPublicKeyBytes,
        ) {
            let resource = ResourceBuilder::fungible()
                .with_token_symbol(&token_symbol)
                .build();

            let linking_tags_address = ResourceBuilder::non_fungible()
                .with_token_symbol(&format!("{}_linking", token_symbol))
                .build();
            let chain_rules = ChainRules {
                resource_address: resource,
                linking_tags_address: linking_tags_address,
            };

            // let rules_component = Component::new(chain_rules).create();

            Self::up(owner1.clone(), chain_rules.clone());
            Self::up(owner2.clone(), chain_rules.clone());
            Self::up(owner3.clone(), chain_rules.clone());
            Self::up(owner4.clone(), chain_rules.clone());

            // Component::new(Self {
            //     vault: Vault::from_bucket(coins),
            //     owner,
            //     is_spent: false,
            // })
            // .with_access_rules(AccessRules::allow_all())
            // .create();
            // Self::up(Vault::from_bucket(coins), owner);
        }

        fn up(owner: RistrettoPublicKeyBytes, rules: ChainRules) -> Component<Self> {
            let address_alloc = CallerContext::allocate_component_address();
            Component::new(Self {
                owner,
                chain_rules: rules,
            })
            .with_access_rules(AccessRules::allow_all())
            .with_address_allocation(address_alloc)
            .create()
        }

        // fn down(&mut self) -> Bucket {
        //     if self.is_spent {
        //         panic!("ECheck has already been spent");
        //     }
        //     let b = self.vault.withdraw_all();
        //     self.is_spent = true;
        //     b
        // }

        pub fn spend(
            &self,
            other_input1: ComponentAddress,
            other_input2: ComponentAddress,
            other_input3: ComponentAddress,
            to: RistrettoPublicKeyBytes,
            change_addr: RistrettoPublicKeyBytes,
            linking_tag: RistrettoPublicKeyBytes,
            triptych_proof: Vec<u8>,
        ) -> (Component<Self>, Component<Self>) {
            // check triptych proof
            // todo: confirm balance proof
            // if other_inputs.len() + 1 != num_decoys as usize {
            // panic!("Incorrect number of decoys");
            // }

            let mut input_pub_keys = [other_input1, other_input2, other_input3]
                .iter()
                .map(|address| {
                    let owner: RistrettoPublicKeyBytes =
                        ComponentManager::get(*address).call("get_owner", args![]);

                    let decompressed = CompressedRistretto::from_slice(owner.as_bytes())
                        .unwrap()
                        .decompress()
                        .unwrap();
                    decompressed
                })
                .collect::<Vec<RistrettoPoint>>();

            input_pub_keys.insert(
                0,
                CompressedRistretto::from_slice(self.owner.as_bytes())
                    .unwrap()
                    .decompress()
                    .unwrap(),
            );

            let input_set = Arc::new(InputSet::new(&input_pub_keys));

            let params = Arc::new(Parameters::new(2, 2).unwrap());

            let linking_tag =
                CompressedRistretto::from_slice(linking_tag.as_bytes())?.decompress()?;
            let statement = TriptychStatement::new(&params, &input_set, &decompressed_linking_tag)?;
            let mut transcript = Transcript::new(b"<tx data>");
            let proof = TriptychProof::from_bytes(&triptych_proof).unwrap();
            match proof.verify(&statement, &mut transcript) {
                Ok(_) => {}
                Err(e) => {
                    panic!("Proof verification failed: {:?}", e);
                }
            }
            let bucket =
                res_manager.mint_non_fungible(NonFungibleId::from_u256(linking_tag_u256), &{}, &{});

            let dest = Self::up(to, self.chain_rules.clone());
            let change = Self::up(change_addr, self.chain_rules.clone());

            // Get the resource manager
            let mut res_manager = ResourceManager::get(self.chain_rules.linking_tags_address);

            let mut u256id = [0u8; 32];

            u256id.copy_from_slice(&linking_tag.as_bytes()[..32]);

            // Mint the NFT
            let bucket = res_manager.mint_non_fungible(NonFungibleId::from_u256(u256id), &{}, &{});

            bucket.burn();

            (dest, change)
        }

        pub fn get_owner(&self) -> RistrettoPublicKeyBytes {
            self.owner.clone()
        }
    }
}
