// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::{InputSelection, Requirement};
use crate::{
    block::address::Address,
    error::{Error, Result},
    secret::types::InputSigningData,
};

fn has_ed25519_address(input: &InputSigningData, address: &Address) -> bool {
    // TODO could also be in state/governor?
    if let Some(unlock_conditions) = input.output.unlock_conditions() {
        if let Some(address_unlock_condition) = unlock_conditions.address() {
            address_unlock_condition.address() == address
        } else {
            false
        }
    } else {
        false
    }
}

impl InputSelection {
    fn fulfill_ed25519_address_requirement(
        &mut self,
        address: Address,
    ) -> Result<(Vec<InputSigningData>, Option<Requirement>)> {
        // Checks if the requirement is already fulfilled.
        if self
            .selected_inputs
            .iter()
            .any(|input| has_ed25519_address(input, &address))
        {
            return Ok((Vec::new(), None));
        }

        // Checks if the requirement can be fulfilled.

        // TODO bit dumb atm, need to add more possible strategies.

        // TODO check that the enumeration index is kept original and not filtered.
        // Tries to find a basic output first.
        let index = if let Some((index, _)) = self
            .available_inputs
            .iter()
            .enumerate()
            .find(|(_, input)| input.output.is_basic() && has_ed25519_address(input, &address))
        {
            Some(index)
        } else {
            // TODO any preference between alias and NFT?
            // If no basic output has been found, tries the other kinds of output.
            self.available_inputs.iter().enumerate().find_map(|(index, input)| {
                if !input.output.is_basic() && has_ed25519_address(input, &address) {
                    Some(index)
                } else {
                    None
                }
            })
        };

        match index {
            Some(index) => Ok((vec![self.available_inputs.swap_remove(index)], None)),
            None => Err(Error::UnfulfillableRequirement(Requirement::Sender(address))),
        }
    }

    /// Fulfills a sender requirement.
    pub(crate) fn fulfill_sender_requirement(
        &mut self,
        address: Address,
    ) -> Result<(Vec<InputSigningData>, Option<Requirement>)> {
        match address {
            Address::Ed25519(_) => self.fulfill_ed25519_address_requirement(address),
            Address::Alias(alias_address) => {
                match self.fulfill_alias_requirement(alias_address.into_alias_id(), true) {
                    Ok(res) => Ok(res),
                    Err(Error::UnfulfillableRequirement(Requirement::Alias(_, _))) => {
                        Err(Error::UnfulfillableRequirement(Requirement::Sender(address)))
                    }
                    Err(e) => Err(e),
                }
            }
            Address::Nft(nft_address) => match self.fulfill_nft_requirement(nft_address.into_nft_id()) {
                Ok(res) => Ok(res),
                Err(Error::UnfulfillableRequirement(Requirement::Nft(_))) => {
                    Err(Error::UnfulfillableRequirement(Requirement::Sender(address)))
                }
                Err(e) => Err(e),
            },
        }
    }
}
