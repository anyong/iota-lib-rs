// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::Requirement;
use crate::{
    block::output::{AliasId, Output},
    error::{Error, Result},
    secret::types::InputSigningData,
};

/// Tries to fulfill an alias requirement by selecting the appropriate alias output from the available inputs.
pub(crate) fn fulfill_alias_requirement(
    alias_id: &AliasId,
    available_inputs: &mut Vec<InputSigningData>,
    selected_inputs: &[InputSigningData],
    _outputs: &[Output],
) -> Result<Vec<InputSigningData>> {
    fn predicate(input: &InputSigningData, alias_id: &AliasId) -> bool {
        if let Output::Alias(alias_output) = &input.output {
            &alias_output.alias_id_non_null(input.output_id()) == alias_id
        } else {
            false
        }
    }

    // Checks if the requirement is already fulfilled.
    if selected_inputs
        .iter()
        .find(|input| predicate(input, alias_id))
        .is_some()
    {
        return Ok(Vec::new());
    }

    // Checks if the requirement can be fulfilled.
    {
        let index = available_inputs.iter().position(|input| predicate(input, alias_id));

        match index {
            Some(index) => Ok(vec![available_inputs.swap_remove(index)]),
            None => Err(Error::UnfulfillableRequirement(Requirement::Alias(*alias_id))),
        }
    }
}
