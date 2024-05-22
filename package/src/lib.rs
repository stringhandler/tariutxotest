use tari_template_lib::prelude::*;

#[template]
mod echeck_template {
    use super::*;

    pub struct ECheck {
        vault: Vault,
        owner: RistrettoPublicKeyBytes,
        is_spent: bool,
    }

    impl ECheck {
        pub fn mint_first(
            initial_supply: Amount,
            token_symbol: String,
            owner: RistrettoPublicKeyBytes,
        ) {
            let coins = ResourceBuilder::fungible()
                .with_token_symbol(&token_symbol)
                .initial_supply(initial_supply)
                .build_bucket();

            // Component::new(Self {
            //     vault: Vault::from_bucket(coins),
            //     owner,
            //     is_spent: false,
            // })
            // .with_access_rules(AccessRules::allow_all())
            // .create();
            Self::up(Vault::from_bucket(coins), owner);
        }

        fn up(coins: Vault, owner: RistrettoPublicKeyBytes) -> Component<Self> {
            let address_alloc = CallerContext::allocate_component_address();
            Component::new(Self {
                vault: coins,
                owner,
                is_spent: false,
            })
            .with_access_rules(AccessRules::allow_all())
            .with_address_allocation(address_alloc)
            .create()
        }

        fn down(&mut self) -> Bucket {
            if self.is_spent {
                panic!("ECheck has already been spent");
            }
            let b = self.vault.withdraw_all();
            self.is_spent = true;
            b
        }

        pub fn spend(
            &mut self,
            other_inputs: Vec<ComponentAddress>,
            amount: Amount,
            to: RistrettoPublicKeyBytes,
        ) -> (Component<Self>, Component<Self>) {
            if self.is_spent {
                panic!("ECheck has already been spent");
            }
            let mut result = Vault::new_empty(self.vault.resource_address());
            let mut change = Vault::new_empty(self.vault.resource_address());
            let mut remaining_to_filled = amount;
            let mut bucket = self.down();
            if bucket.amount() > amount {
                let split = bucket.take(amount);
                result.deposit(split);
                change.deposit(bucket);
                remaining_to_filled = Amount::new(0);
            } else {
                // result.deposit(bucket);
                remaining_to_filled = amount - bucket.amount();
                result.deposit(bucket);
            }

            for input in other_inputs {
                let mut other = ComponentManager::get(input);
                let mut bucket: Bucket = other.call("down", args![]);
                if bucket.amount() > remaining_to_filled {
                    let split = bucket.take(amount);
                    result.deposit(split);
                    change.deposit(bucket);
                    remaining_to_filled = Amount::new(0);
                } else {
                    remaining_to_filled -= bucket.amount();
                    result.deposit(bucket);
                }
            }

            if remaining_to_filled > Amount::new(0) {
                panic!("Not enough funds to spend");
            }
            (Self::up(result, to), Self::up(change, self.owner))
        }
    }
}
