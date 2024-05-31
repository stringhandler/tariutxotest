use tari_template_lib::models::ComponentKey;
use tari_template_lib::models::EntityId;
use tari_template_lib::models::ObjectKey;
use tari_template_lib::prelude::*;

#[template]
mod unique_slot_template {
    use super::*;

    pub struct UniqueSlot {
        id: RistrettoPublicKeyBytes,
    }

    impl UniqueSlot {
        pub fn create(id: RistrettoPublicKeyBytes) -> Component<Self> {
            Component::new(Self { id: id.clone() })
                .with_owner_rule(OwnerRule::ByPublicKey(id))
                .create()
        }
    }
}
