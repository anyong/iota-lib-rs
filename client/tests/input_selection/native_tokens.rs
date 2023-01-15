// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::str::FromStr;

use iota_client::{
    api::input_selection::new::InputSelection,
    block::{address::Address, output::TokenId, protocol::protocol_parameters},
    Error,
};
use primitive_types::U256;

use crate::input_selection::{
    build_inputs, build_outputs, unsorted_eq, Build::Basic, BECH32_ADDRESS, TOKEN_ID_1, TOKEN_ID_2,
};

#[test]
fn two_native_tokens_one_needed() {
    let protocol_parameters = protocol_parameters();

    let inputs = build_inputs(vec![
        Basic(1_000_000, BECH32_ADDRESS, Some(vec![(TOKEN_ID_1, 100)]), None),
        Basic(
            1_000_000,
            BECH32_ADDRESS,
            Some(vec![(TOKEN_ID_1, 100), (TOKEN_ID_2, 100)]),
            None,
        ),
    ]);
    let outputs = build_outputs(vec![Basic(
        1_000_000,
        BECH32_ADDRESS,
        Some(vec![(TOKEN_ID_1, 150)]),
        None,
    )]);

    let selected = InputSelection::new(inputs.clone(), outputs.clone(), protocol_parameters)
        .select()
        .unwrap();

    assert!(unsorted_eq(&selected.inputs, &inputs));
    assert_eq!(selected.outputs.len(), 2);
    assert!(selected.outputs.contains(&outputs[0]));
    selected.outputs.iter().for_each(|output| {
        if !outputs.contains(output) {
            assert!(output.is_basic());
            assert_eq!(output.amount(), 1_000_000);
            assert_eq!(output.as_basic().native_tokens().len(), 2);
            assert_eq!(
                output
                    .as_basic()
                    .native_tokens()
                    .get(&TokenId::from_str(TOKEN_ID_1).unwrap())
                    .unwrap()
                    .amount(),
                U256::from(50),
            );
            assert_eq!(
                output
                    .as_basic()
                    .native_tokens()
                    .get(&TokenId::from_str(TOKEN_ID_2).unwrap())
                    .unwrap()
                    .amount(),
                U256::from(100),
            );
            assert_eq!(output.as_basic().unlock_conditions().len(), 1);
            assert_eq!(output.as_basic().features().len(), 0);
            assert_eq!(
                *output.as_basic().address(),
                Address::try_from_bech32(BECH32_ADDRESS).unwrap().1
            );
        }
    });
}

#[test]
fn two_native_tokens_both_needed_plus_remainder() {
    let protocol_parameters = protocol_parameters();

    let inputs = build_inputs(vec![
        Basic(1_000_000, BECH32_ADDRESS, Some(vec![(TOKEN_ID_1, 100)]), None),
        Basic(
            1_000_000,
            BECH32_ADDRESS,
            Some(vec![(TOKEN_ID_1, 100), (TOKEN_ID_2, 100)]),
            None,
        ),
    ]);
    let outputs = build_outputs(vec![Basic(
        1_000_000,
        BECH32_ADDRESS,
        Some(vec![(TOKEN_ID_1, 150), (TOKEN_ID_2, 100)]),
        None,
    )]);

    let selected = InputSelection::new(inputs.clone(), outputs.clone(), protocol_parameters)
        .select()
        .unwrap();

    assert!(unsorted_eq(&selected.inputs, &inputs));
    assert_eq!(selected.outputs.len(), 2);
    assert!(selected.outputs.contains(&outputs[0]));
    selected.outputs.iter().for_each(|output| {
        if !outputs.contains(output) {
            assert!(output.is_basic());
            assert_eq!(output.amount(), 1_000_000);
            assert_eq!(output.as_basic().native_tokens().len(), 1);
            assert_eq!(
                output
                    .as_basic()
                    .native_tokens()
                    .get(&TokenId::from_str(TOKEN_ID_1).unwrap())
                    .unwrap()
                    .amount(),
                U256::from(50),
            );
            assert_eq!(output.as_basic().unlock_conditions().len(), 1);
            assert_eq!(output.as_basic().features().len(), 0);
            assert_eq!(
                *output.as_basic().address(),
                Address::try_from_bech32(BECH32_ADDRESS).unwrap().1
            );
        }
    });
}

#[test]
fn three_inputs_two_needed_plus_remainder() {
    let protocol_parameters = protocol_parameters();

    let inputs = build_inputs(vec![
        Basic(1_000_000, BECH32_ADDRESS, Some(vec![(TOKEN_ID_1, 100)]), None),
        Basic(1_000_000, BECH32_ADDRESS, Some(vec![(TOKEN_ID_1, 100)]), None),
        Basic(1_000_000, BECH32_ADDRESS, Some(vec![(TOKEN_ID_1, 100)]), None),
    ]);
    let outputs = build_outputs(vec![Basic(
        1_000_000,
        BECH32_ADDRESS,
        Some(vec![(TOKEN_ID_1, 120)]),
        None,
    )]);

    let selected = InputSelection::new(inputs, outputs.clone(), protocol_parameters)
        .select()
        .unwrap();

    assert_eq!(selected.inputs.len(), 2);
    assert_eq!(selected.outputs.len(), 2);
    assert!(selected.outputs.contains(&outputs[0]));
    selected.outputs.iter().for_each(|output| {
        if !outputs.contains(output) {
            assert!(output.is_basic());
            assert_eq!(output.amount(), 1_000_000);
            assert_eq!(output.as_basic().native_tokens().len(), 1);
            assert_eq!(
                output
                    .as_basic()
                    .native_tokens()
                    .get(&TokenId::from_str(TOKEN_ID_1).unwrap())
                    .unwrap()
                    .amount(),
                U256::from(80),
            );
            assert_eq!(output.as_basic().unlock_conditions().len(), 1);
            assert_eq!(output.as_basic().features().len(), 0);
            assert_eq!(
                *output.as_basic().address(),
                Address::try_from_bech32(BECH32_ADDRESS).unwrap().1
            );
        }
    });
}

#[test]
fn three_inputs_two_needed_no_remainder() {
    let protocol_parameters = protocol_parameters();

    let inputs = build_inputs(vec![
        Basic(1_000_000, BECH32_ADDRESS, Some(vec![(TOKEN_ID_1, 100)]), None),
        Basic(1_000_000, BECH32_ADDRESS, Some(vec![(TOKEN_ID_1, 100)]), None),
        Basic(1_000_000, BECH32_ADDRESS, Some(vec![(TOKEN_ID_1, 100)]), None),
    ]);
    let outputs = build_outputs(vec![Basic(
        2_000_000,
        BECH32_ADDRESS,
        Some(vec![(TOKEN_ID_1, 200)]),
        None,
    )]);

    let selected = InputSelection::new(inputs, outputs.clone(), protocol_parameters)
        .select()
        .unwrap();

    assert_eq!(selected.inputs.len(), 2);
    assert_eq!(selected.outputs, outputs);
}

#[test]
fn insufficient_native_tokens_one_input() {
    let protocol_parameters = protocol_parameters();

    let inputs = build_inputs(vec![Basic(
        1_000_000,
        BECH32_ADDRESS,
        Some(vec![(TOKEN_ID_1, 100)]),
        None,
    )]);
    let outputs = build_outputs(vec![Basic(
        1_000_000,
        BECH32_ADDRESS,
        Some(vec![(TOKEN_ID_1, 150)]),
        None,
    )]);

    let selected = InputSelection::new(inputs, outputs, protocol_parameters).select();

    assert!(matches!(
        selected,
        Err(Error::InsufficientNativeTokenAmount {
            token_id,
            found,
            required,
        }) if token_id == TokenId::from_str(TOKEN_ID_1).unwrap() && found == U256::from(100) && required == U256::from(150)));
}

#[test]
fn insufficient_native_tokens_three_inputs() {
    let protocol_parameters = protocol_parameters();

    let inputs = build_inputs(vec![
        Basic(1_000_000, BECH32_ADDRESS, Some(vec![(TOKEN_ID_1, 100)]), None),
        Basic(1_000_000, BECH32_ADDRESS, Some(vec![(TOKEN_ID_1, 100)]), None),
        Basic(1_000_000, BECH32_ADDRESS, Some(vec![(TOKEN_ID_1, 100)]), None),
    ]);
    let outputs = build_outputs(vec![Basic(
        1_000_000,
        BECH32_ADDRESS,
        Some(vec![(TOKEN_ID_1, 301)]),
        None,
    )]);

    let selected = InputSelection::new(inputs, outputs, protocol_parameters).select();

    assert!(matches!(
        selected,
        Err(Error::InsufficientNativeTokenAmount {
            token_id,
            found,
            required,
        }) if token_id == TokenId::from_str(TOKEN_ID_1).unwrap() && found == U256::from(300) && required == U256::from(301)));
}
