// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub(crate) mod burn;
pub(crate) mod remainder;
pub(crate) mod requirement;
pub(crate) mod transition;

use std::collections::HashSet;

use self::requirement::alias::is_alias_transition;
pub use self::{
    burn::{Burn, BurnDto},
    requirement::Requirement,
};
use crate::{
    api::types::RemainderData,
    block::{
        address::{Address, AliasAddress, Ed25519Address, NftAddress},
        output::{AliasTransition, ChainId, Output, OutputId},
        protocol::ProtocolParameters,
    },
    error::{Error, Result},
    secret::types::InputSigningData,
};

// TODO should ISA have its own error type? At least review errors.
// TODO make methods actually take self? There was a mut issue.

/// Working state for the input selection algorithm.
pub struct InputSelection {
    available_inputs: Vec<InputSigningData>,
    // TODO option needed ?
    required_inputs: Option<HashSet<OutputId>>,
    forbidden_inputs: HashSet<OutputId>,
    selected_inputs: Vec<InputSigningData>,
    outputs: Vec<Output>,
    addresses: HashSet<Address>,
    burn: Option<Burn>,
    remainder_address: Option<Address>,
    protocol_parameters: ProtocolParameters,
    timestamp: u32,
    requirements: Vec<Requirement>,
    automatically_transitioned: HashSet<ChainId>,
}

/// Result of the input selection algorithm.
#[derive(Clone, Debug)]
pub struct Selected {
    /// Selected inputs.
    pub inputs: Vec<InputSigningData>,
    /// Provided and created outputs.
    pub outputs: Vec<Output>,
    /// Remainder, if there was one.
    pub remainder: Option<RemainderData>,
}

impl InputSelection {
    fn required_alias_nft_addresses(&self, input: &InputSigningData) -> Result<Option<Requirement>> {
        // TODO burn?
        let alias_transition = is_alias_transition(input, &self.outputs).map(|(alias_transition, _)| alias_transition);
        let (required_address, _) =
            input
                .output
                .required_and_unlocked_address(self.timestamp, input.output_id(), alias_transition)?;

        match required_address {
            Address::Alias(alias_address) => Ok(Some(Requirement::Alias(
                *alias_address.alias_id(),
                AliasTransition::State,
            ))),
            Address::Nft(nft_address) => Ok(Some(Requirement::Nft(*nft_address.nft_id()))),
            _ => Ok(None),
        }
    }

    fn select_input(&mut self, input: InputSigningData, alias_transition: Option<AliasTransition>) -> Result<()> {
        log::debug!("Selecting input {:?}", input.output_id());

        if let Some(output) = self.transition_input(&input, alias_transition)? {
            // No need to check for `outputs_requirements` because
            // - the sender feature doesn't need to be verified as it has been removed
            // - the issuer feature doesn't need to be verified as the chain is not new
            // - input doesn't need to be checked for as we just transitioned it
            // - foundry alias requirement should have been met already by a prior `required_alias_nft_addresses`
            self.outputs.push(output);
        }

        if let Some(requirement) = self.required_alias_nft_addresses(&input)? {
            log::debug!("Adding {requirement:?} from input {:?}", input.output_id());
            self.requirements.push(requirement);
        }

        self.selected_inputs.push(input);

        Ok(())
    }

    // TODO rename
    fn init(&mut self) -> Result<()> {
        // Adds an initial amount requirement.
        self.requirements.push(Requirement::Amount);
        // Adds an initial native tokens requirement.
        self.requirements.push(Requirement::NativeTokens);

        // Removes forbidden inputs from available inputs.
        self.available_inputs
            .retain(|input| !self.forbidden_inputs.contains(input.output_id()));

        // The `take` avoids a mutable borrow compilation issue without having to clone the required inputs.
        // TODO could be reworked by having select_input not taking mut.
        if let Some(required_inputs) = self.required_inputs.take() {
            for required_input in required_inputs.into_iter() {
                // Checks that required input is not forbidden.
                if self.forbidden_inputs.contains(&required_input) {
                    return Err(Error::RequiredInputIsForbidden(required_input));
                }

                // Checks that required input is available.
                match self
                    .available_inputs
                    .iter()
                    .position(|input| input.output_id() == &required_input)
                {
                    Some(index) => {
                        // Removes required input from available inputs.
                        let input = self.available_inputs.swap_remove(index);

                        // Selects required input.
                        self.select_input(input, None)?
                    }
                    None => return Err(Error::RequiredInputIsNotAvailable(required_input)),
                }
            }
        }

        // Gets requirements from outputs.
        // TODO this may re-evaluate outputs added by inputs
        self.outputs_requirements();

        // Gets requirements from burn.
        self.burn_requirements()?;

        Ok(())
    }

    /// Creates a new [`InputSelection`].
    pub fn new(
        available_inputs: Vec<InputSigningData>,
        outputs: Vec<Output>,
        addresses: Vec<Address>,
        protocol_parameters: ProtocolParameters,
    ) -> Self {
        let mut addresses = HashSet::from_iter(addresses);

        addresses.extend(available_inputs.iter().filter_map(|input| match &input.output {
            Output::Alias(output) => Some(Address::Alias(AliasAddress::from(
                output.alias_id_non_null(input.output_id()),
            ))),
            Output::Nft(output) => Some(Address::Nft(NftAddress::from(
                output.nft_id_non_null(input.output_id()),
            ))),
            _ => None,
        }));

        Self {
            available_inputs,
            required_inputs: None,
            forbidden_inputs: HashSet::new(),
            selected_inputs: Vec::new(),
            outputs,
            addresses,
            burn: None,
            remainder_address: None,
            protocol_parameters,
            timestamp: instant::SystemTime::now()
                .duration_since(instant::SystemTime::UNIX_EPOCH)
                .expect("time went backwards")
                .as_secs() as u32,
            requirements: Vec::new(),
            automatically_transitioned: HashSet::new(),
        }
    }

    /// Sets the required inputs of an [`InputSelection`].
    pub fn required_inputs(mut self, inputs: HashSet<OutputId>) -> Self {
        self.required_inputs.replace(inputs);
        self
    }

    /// Sets the forbidden inputs of an [`InputSelection`].
    pub fn forbidden_inputs(mut self, inputs: HashSet<OutputId>) -> Self {
        self.forbidden_inputs = inputs;
        self
    }

    /// Sets the burn of an [`InputSelection`].
    pub fn burn(mut self, burn: Burn) -> Self {
        self.burn.replace(burn);
        self
    }

    /// Sets the remainder address of an [`InputSelection`].
    pub fn remainder_address(mut self, address: Address) -> Self {
        self.remainder_address.replace(address);
        self
    }

    /// Sets the timestamp of an [`InputSelection`].
    pub fn timestamp(mut self, timestamp: u32) -> Self {
        self.timestamp = timestamp;
        self
    }

    fn filter_inputs(&mut self) {
        self.available_inputs.retain(|input| {
            // Keep alias outputs because at this point we do not know if a state or governor address will be required.
            if input.output.is_alias() {
                return true;
            }
            // Filter out non basic/foundry/nft outputs.
            else if !input.output.is_basic() && !input.output.is_foundry() && !input.output.is_nft() {
                return false;
            }

            // PANIC: safe to unwrap as non basic/alias/foundry/nft outputs are already filtered out.
            let unlock_conditions = input.output.unlock_conditions().unwrap();

            if unlock_conditions.is_time_locked(self.timestamp) {
                return false;
            }

            let required_address = input
                .output
                // Alias transition is irrelevant here as we keep aliases anyway.
                .required_and_unlocked_address(self.timestamp, input.output_id(), None)
                // PANIC: safe to unwrap as non basic/alias/foundry/nft outputs are already filtered out.
                .unwrap()
                .0;

            self.addresses.contains(&required_address)
        })
    }

    // Inputs need to be sorted before signing, because the reference unlock conditions can only reference a lower index
    pub(crate) fn sort_input_signing_data(inputs: Vec<InputSigningData>) -> crate::Result<Vec<InputSigningData>> {
        // filter for ed25519 address first, safe to unwrap since we encoded it before
        let (mut sorted_inputs, alias_nft_address_inputs): (Vec<InputSigningData>, Vec<InputSigningData>) = inputs
            .into_iter()
            // PANIC: safe to unwrap as we encoded the address before
            .partition(|input| {
                Address::try_from_bech32(&input.bech32_address).unwrap().1.kind() == Ed25519Address::KIND
            });

        for input in alias_nft_address_inputs {
            let input_address = Address::try_from_bech32(&input.bech32_address);
            match sorted_inputs.iter().position(|input_signing_data| match input_address {
                Ok((_, unlock_address)) => match unlock_address {
                    Address::Alias(unlock_address) => {
                        if let Output::Alias(alias_output) = &input_signing_data.output {
                            *unlock_address.alias_id() == alias_output.alias_id_non_null(input_signing_data.output_id())
                        } else {
                            false
                        }
                    }
                    Address::Nft(unlock_address) => {
                        if let Output::Nft(nft_output) = &input_signing_data.output {
                            *unlock_address.nft_id() == nft_output.nft_id_non_null(input_signing_data.output_id())
                        } else {
                            false
                        }
                    }
                    _ => false,
                },
                _ => false,
            }) {
                Some(position) => {
                    // Insert after the output we need
                    sorted_inputs.insert(position + 1, input);
                }
                None => {
                    // insert before address
                    let alias_or_nft_address = match &input.output {
                        Output::Alias(alias_output) => Some(Address::Alias(AliasAddress::new(
                            alias_output.alias_id_non_null(input.output_id()),
                        ))),
                        Output::Nft(nft_output) => Some(Address::Nft(NftAddress::new(
                            nft_output.nft_id_non_null(input.output_id()),
                        ))),
                        _ => None,
                    };

                    if let Some(alias_or_nft_address) = alias_or_nft_address {
                        // Check for existing outputs for this address, and insert before
                        match sorted_inputs.iter().position(|input_signing_data| {
                            Address::try_from_bech32(&input_signing_data.bech32_address)
                                .expect("safe to unwrap, we encoded it before")
                                .1
                                == alias_or_nft_address
                        }) {
                            Some(position) => {
                                // Insert before the output with this address required for unlocking
                                sorted_inputs.insert(position, input);
                            }
                            // just push output
                            None => sorted_inputs.push(input),
                        }
                    } else {
                        // just push basic or foundry output
                        sorted_inputs.push(input);
                    }
                }
            }
        }

        Ok(sorted_inputs)
    }

    /// Selects inputs that meet the requirements of the outputs to satisfy the semantic validation of the overall
    /// transaction. Also creates a remainder output and chain transition outputs if required.
    pub fn select(mut self) -> Result<Selected> {
        self.filter_inputs();

        if self.available_inputs.is_empty() {
            return Err(Error::NoAvailableInputsProvided);
        }
        if self.outputs.is_empty() && self.burn.is_none() {
            return Err(Error::NoOutputsProvided);
        }

        // Creates the initial state, selected inputs and requirements, based on the provided outputs.
        self.init()?;

        // Process all the requirements until there are no more.
        while let Some(requirement) = self.requirements.pop() {
            // Fulfill the requirement.
            let inputs = self.fulfill_requirement(requirement)?;

            // Select suggested inputs.
            for (input, alias_transition) in inputs {
                self.select_input(input, alias_transition)?;
            }
        }

        let (remainder, storage_deposit_returns) = self.remainder_and_storage_deposit_return_outputs()?;

        if let Some(remainder) = &remainder {
            self.outputs.push(remainder.output.clone());
        }

        self.outputs.extend(storage_deposit_returns);

        Ok(Selected {
            inputs: Self::sort_input_signing_data(self.selected_inputs)?,
            outputs: self.outputs,
            remainder,
        })
    }
}
