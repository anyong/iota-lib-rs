// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::str::FromStr;

use iota_client::{
    api::input_selection::new::{Burn, InputSelection, Requirement},
    block::{
        output::{AliasId, Output},
        protocol::protocol_parameters,
    },
    Error, Result,
};
use iota_types::block::output::{NativeToken, SimpleTokenScheme, TokenId};
use primitive_types::U256;

use crate::input_selection::{
    build_alias_output, build_foundry_output, build_input_signing_data_alias_outputs,
    build_input_signing_data_foundry_outputs, build_input_signing_data_most_basic_outputs, build_most_basic_output,
};

#[test]
fn input_selection_alias() -> Result<()> {
    let protocol_parameters = protocol_parameters();

    let alias_id_0 = AliasId::from_str("0x0000000000000000000000000000000000000000000000000000000000000000").unwrap();
    let alias_id_1 = AliasId::from_str("0x1111111111111111111111111111111111111111111111111111111111111111").unwrap();
    let bech32_address = "rms1qr2xsmt3v3eyp2ja80wd2sq8xx0fslefmxguf7tshzezzr5qsctzc2f5dg6";

    // input alias == output alias
    let inputs = build_input_signing_data_alias_outputs(vec![(alias_id_1, bech32_address, 1_000_000)]);
    let outputs = vec![build_alias_output(alias_id_1, bech32_address, 1_000_000)];
    let selected_transaction_data = InputSelection::build(outputs, inputs.clone(), protocol_parameters.clone())
        .finish()
        .select()?;

    assert_eq!(selected_transaction_data.0, inputs);

    // output amount > input amount
    let inputs = build_input_signing_data_alias_outputs(vec![(alias_id_1, bech32_address, 1_000_000)]);
    let outputs = vec![build_most_basic_output(bech32_address, 2_000_000)];

    match InputSelection::build(outputs, inputs, protocol_parameters.clone())
        .finish()
        .select()
    {
        Err(Error::NotEnoughBalance {
            found: 1_000_000,
            // Amount we want to send + storage deposit for alias remainder
            required: 2_251_500,
        }) => {}
        _ => panic!("Should return NotEnoughBalance"),
    }

    // basic output with alias as input
    let inputs = build_input_signing_data_alias_outputs(vec![(alias_id_1, bech32_address, 2_251_500)]);
    let outputs = vec![build_most_basic_output(bech32_address, 2_000_000)];
    let selected_transaction_data = InputSelection::build(outputs, inputs.clone(), protocol_parameters.clone())
        .finish()
        .select()?;

    // basic output + alias remainder
    assert_eq!(selected_transaction_data.1.len(), 2);

    // mint alias
    let inputs = build_input_signing_data_most_basic_outputs(vec![(bech32_address, 2_000_000)]);
    let outputs = vec![build_alias_output(alias_id_0, bech32_address, 1_000_000)];
    let selected_transaction_data = InputSelection::build(outputs, inputs.clone(), protocol_parameters.clone())
        .finish()
        .select()?;

    // One output should be added for the remainder
    assert_eq!(selected_transaction_data.1.len(), 2);
    // Output contains the new minted alias id
    assert!(selected_transaction_data.1.iter().any(|output| {
        if let Output::Alias(alias_output) = output {
            *alias_output.alias_id() == alias_id_0
        } else {
            false
        }
    }));

    // burn alias
    let inputs = build_input_signing_data_alias_outputs(vec![(alias_id_1, bech32_address, 2_000_000)]);
    let outputs = vec![build_most_basic_output(bech32_address, 2_000_000)];
    let selected_transaction_data = InputSelection::build(outputs, inputs.clone(), protocol_parameters.clone())
        .burn(Burn::new().add_alias(alias_id_1))
        .finish()
        .select()?;

    // No remainder
    assert_eq!(selected_transaction_data.1.len(), 1);
    // Output is a basic output
    assert!(matches!(selected_transaction_data.1[0], Output::Basic(_)));

    // not enough storage deposit for remainder
    let inputs = build_input_signing_data_alias_outputs(vec![(alias_id_1, bech32_address, 1_000_001)]);
    let outputs = vec![build_alias_output(alias_id_1, bech32_address, 1_000_000)];

    match InputSelection::build(outputs, inputs, protocol_parameters.clone())
        .finish()
        .select()
    {
        Err(Error::BlockError(iota_types::block::Error::InsufficientStorageDepositAmount {
            amount: 1,
            required: 213000,
        })) => {}
        _ => panic!("Should return InsufficientStorageDepositAmount"),
    }

    // missing input for output alias
    let inputs = build_input_signing_data_most_basic_outputs(vec![(bech32_address, 1_000_000)]);
    let outputs = vec![build_alias_output(alias_id_1, bech32_address, 1_000_000)];

    match InputSelection::build(outputs, inputs, protocol_parameters.clone())
        .finish()
        .select()
    {
        Err(Error::UnfulfillableRequirement(req)) => {
            assert_eq!(req, Requirement::Alias(alias_id_1));
        }
        _ => panic!("Should return missing alias input"),
    }

    ////////////////////////////////////////////////////////////////
    // Foundry
    ////////////////////////////////////////////////////////////////

    // missing input alias for foundry
    let inputs = build_input_signing_data_most_basic_outputs(vec![(bech32_address, 1_000_000)]);
    let outputs = vec![build_foundry_output(
        alias_id_1,
        1_000_000,
        SimpleTokenScheme::new(U256::from(0), U256::from(0), U256::from(10)).unwrap(),
        None,
    )];

    match InputSelection::build(outputs, inputs, protocol_parameters.clone())
        .finish()
        .select()
    {
        Err(Error::UnfulfillableRequirement(req)) => {
            assert_eq!(req, Requirement::Alias(alias_id_1));
        }
        _ => panic!("Should return missing alias input"),
    }

    // existing input alias for foundry alias
    let inputs = build_input_signing_data_alias_outputs(vec![(alias_id_1, bech32_address, 1251500)]);
    let outputs = vec![build_foundry_output(
        alias_id_1,
        1_000_000,
        SimpleTokenScheme::new(U256::from(0), U256::from(0), U256::from(10)).unwrap(),
        None,
    )];
    let selected_transaction_data = InputSelection::build(outputs, inputs.clone(), protocol_parameters.clone())
        .finish()
        .select()?;

    // Alias next state + foundry
    println!("{:?}", selected_transaction_data.1);
    assert_eq!(selected_transaction_data.1.len(), 2);
    // Alias state index is increased
    selected_transaction_data.1.iter().for_each(|output| {
        if let Output::Alias(alias_output) = &output {
            // Input alias has index 0, output should have index 1
            assert_eq!(alias_output.state_index(), 1);
        }
    });

    // minted native tokens in new remainder
    let inputs = build_input_signing_data_alias_outputs(vec![(alias_id_1, bech32_address, 2251500)]);
    let outputs = vec![build_foundry_output(
        alias_id_1,
        1_000_000,
        SimpleTokenScheme::new(U256::from(10), U256::from(0), U256::from(10)).unwrap(),
        None,
    )];
    let selected_transaction_data = InputSelection::build(outputs, inputs.clone(), protocol_parameters.clone())
        .finish()
        .select()?;

    println!("TEST H");

    // Alias next state + foundry + basic output with native tokens
    assert_eq!(selected_transaction_data.1.len(), 3);
    // Alias state index is increased
    selected_transaction_data.1.iter().for_each(|output| {
        if let Output::Alias(alias_output) = &output {
            // Input alias has index 0, output should have index 1
            assert_eq!(alias_output.state_index(), 1);
        }
        if let Output::Basic(basic_output) = &output {
            // Basic output remainder has the minted native tokens
            assert_eq!(basic_output.native_tokens().first().unwrap().amount(), U256::from(10));
        }
    });

    // melting native tokens
    let mut inputs = build_input_signing_data_alias_outputs(vec![(alias_id_1, bech32_address, 1_000_000)]);
    inputs.extend(build_input_signing_data_foundry_outputs(vec![(
        alias_id_1,
        1_000_000,
        SimpleTokenScheme::new(U256::from(10), U256::from(0), U256::from(10)).unwrap(),
        Some(
            NativeToken::new(
                TokenId::from_str("0x0811111111111111111111111111111111111111111111111111111111111111110000000000")
                    .unwrap(),
                U256::from(10),
            )
            .unwrap(),
        ),
    )]));
    let outputs = vec![build_foundry_output(
        alias_id_1,
        1_000_000,
        // Melt 5 native tokens
        SimpleTokenScheme::new(U256::from(10), U256::from(5), U256::from(10)).unwrap(),
        None,
    )];
    let selected_transaction_data = InputSelection::build(outputs, inputs.clone(), protocol_parameters.clone())
        .finish()
        .select()?;

    println!("TEST I");

    // Alias next state + foundry + basic output with native tokens
    assert_eq!(selected_transaction_data.1.len(), 3);
    // Alias state index is increased
    selected_transaction_data.1.iter().for_each(|output| {
        if let Output::Alias(alias_output) = &output {
            // Input alias has index 0, output should have index 1
            assert_eq!(alias_output.state_index(), 1);
        }
        if let Output::Basic(basic_output) = &output {
            // Basic output remainder has the remaining native tokens
            assert_eq!(basic_output.native_tokens().first().unwrap().amount(), U256::from(5));
        }
    });

    // Destroy foundry
    let mut inputs = build_input_signing_data_alias_outputs(vec![(alias_id_1, bech32_address, 50300)]);
    inputs.extend(build_input_signing_data_foundry_outputs(vec![(
        alias_id_1,
        52800,
        SimpleTokenScheme::new(U256::from(10), U256::from(10), U256::from(10)).unwrap(),
        None,
    )]));
    // Alias output gets the amount from the foundry output added
    let outputs = vec![build_alias_output(alias_id_1, bech32_address, 103100)];
    let selected_transaction_data = InputSelection::build(outputs, inputs.clone(), protocol_parameters.clone())
        .finish()
        .select()?;

    println!("TEST J");

    // Alias next state
    assert_eq!(selected_transaction_data.1.len(), 1);

    Ok(())
}
