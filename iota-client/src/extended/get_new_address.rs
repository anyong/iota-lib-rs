use crate::error::Result;
use bee_crypto::ternary::sponge::Kerl;
use bee_signing::ternary::{
    wots::{WotsSecurityLevel, WotsSpongePrivateKeyGeneratorBuilder},
    PrivateKey, PrivateKeyGenerator, PublicKey, TernarySeed as Seed,
};
use bee_transaction::bundled::{Address, BundledTransactionField};

use crate::Client;

/// Builder to construct GetNewAddress API
//#[derive(Debug)]
pub struct GenerateNewAddressBuilder<'a> {
    client: &'a Client,
    seed: &'a Seed<Kerl>,
    index: u64,
    security: WotsSecurityLevel,
}

impl<'a> GenerateNewAddressBuilder<'a> {
    pub(crate) fn new(client: &'a Client, seed: &'a Seed<Kerl>) -> Self {
        Self {
            client,
            seed,
            index: 0,
            security: WotsSecurityLevel::Medium,
        }
    }

    /// Set key index to start search at
    pub fn index(mut self, index: u64) -> Self {
        self.index = index;
        self
    }

    /// Set security level
    pub fn security(mut self, security: u8) -> Self {
        self.security = match security {
            1 => WotsSecurityLevel::Low,
            2 => WotsSecurityLevel::Medium,
            3 => WotsSecurityLevel::High,
            _ => panic!("Invalid security level"),
        };
        self
    }

    /// Send GetNewAddress request
    pub async fn generate(self) -> Result<(u64, Address)> {
        let mut index = self.index;

        loop {
            // TODO impl Error trait in iota_signing_preview
            let address = Address::from_inner_unchecked(
                WotsSpongePrivateKeyGeneratorBuilder::<Kerl>::default()
                    .security_level(self.security)
                    .build()
                    .unwrap()
                    .generate_from_seed(self.seed, index)
                    .unwrap()
                    .generate_public_key()
                    .unwrap()
                    .as_trits()
                    .to_owned(),
            );

            if let Ok(false) = self.client.is_address_used(&address).await {
                break Ok((index, address));
            }

            index += 1;
        }
    }
}
