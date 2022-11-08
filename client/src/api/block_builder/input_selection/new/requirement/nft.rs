// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::Requirement;
use crate::{
    block::output::{NftId, Output},
    error::{Error, Result},
    secret::types::InputSigningData,
};

/// Tries to fulfill a nft requirement by selecting the appropriate nft output from the available inputs.
pub(crate) fn fulfill_nft_requirement(
    nft_id: NftId,
    available_inputs: &mut Vec<InputSigningData>,
    selected_inputs: &[InputSigningData],
    _outputs: &[Output],
) -> Result<Vec<InputSigningData>> {
    fn predicate(input: &InputSigningData, nft_id: &NftId) -> bool {
        if let Output::Nft(nft_output) = &input.output {
            &nft_output.nft_id_non_null(input.output_id()) == nft_id
        } else {
            false
        }
    }

    // Checks if the requirement is already fulfilled.
    if selected_inputs.iter().find(|input| predicate(input, &nft_id)).is_some() {
        return Ok(Vec::new());
    }

    // Checks if the requirement can be fulfilled.
    {
        let index = available_inputs.iter().position(|input| predicate(input, &nft_id));

        match index {
            Some(index) => Ok(vec![available_inputs.swap_remove(index)]),
            None => Err(Error::UnfulfillableRequirement(Requirement::Nft(nft_id))),
        }
    }
}
