// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_types::block::output::UnlockCondition;

use super::{fulfill_alias_requirement, fulfill_nft_requirement, Requirement};
use crate::{
    block::address::Address,
    error::{Error, Result},
    secret::types::InputSigningData,
};

fn is_ed25519_address(input: &InputSigningData, address: &Address) -> bool {
    // TODO could also be in state/governor?
    if let Some([UnlockCondition::Address(unlock)]) = input.output.unlock_conditions().map(|u| u.as_ref()) {
        unlock.address() == address
    } else {
        false
    }
}

fn fulfill_ed25519_address_requirement(
    address: Address,
    available_inputs: &mut Vec<InputSigningData>,
    selected_inputs: &[InputSigningData],
) -> Result<Vec<InputSigningData>> {
    // Checks if the requirement is already fulfilled.
    if selected_inputs.iter().any(|input| is_ed25519_address(input, &address)) {
        return Ok(Vec::new());
    }

    // Checks if the requirement can be fulfilled.
    {
        // TODO bit dumb atm, need to add more possible strategies.

        // TODO check that the enumeration index is kept original and not filtered.
        // Tries to find a basic output first.
        let index = if let Some((index, _)) = selected_inputs
            .iter()
            .enumerate()
            .find(|(_, input)| input.output.is_basic() && is_ed25519_address(input, &address))
        {
            Some(index)
        } else {
            // TODO any preference between alias and NFT?
            // If no basic output has been found, tries the other kinds of output.
            available_inputs.iter().enumerate().find_map(|(index, input)| {
                if !input.output.is_basic() && is_ed25519_address(input, &address) {
                    Some(index)
                } else {
                    None
                }
            })
        };

        match index {
            Some(index) => Ok(vec![available_inputs.swap_remove(index)]),
            None => Err(Error::UnfulfillableRequirement(Requirement::Sender(address))),
        }
    }
}

pub(crate) fn fulfill_sender_requirement(
    address: Address,
    available_inputs: &mut Vec<InputSigningData>,
    selected_inputs: &[InputSigningData],
) -> Result<Vec<InputSigningData>> {
    match address {
        Address::Ed25519(_) => fulfill_ed25519_address_requirement(address, available_inputs, selected_inputs),
        Address::Alias(alias_address) => {
            fulfill_alias_requirement(alias_address.into_alias_id(), available_inputs, selected_inputs)
        }
        Address::Nft(nft_address) => {
            fulfill_nft_requirement(nft_address.into_nft_id(), available_inputs, selected_inputs)
        }
    }
}
