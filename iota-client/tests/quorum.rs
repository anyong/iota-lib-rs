mod common;
use crate::common::*;
use iota_bundle_preview::*;
use iota_client::quorum;
use iota_ternary_preview::*;

#[smol_potat::test]
async fn test_get_balances() {
    client_init();
    let _ = quorum::get_balances()
        .addresses(&[Address::from_inner_unchecked(
            TryteBuf::try_from_str(TEST_ADDRESS_0)
                .unwrap()
                .as_trits()
                .encode(),
        )])
        .send()
        .await
        .unwrap();
}

#[smol_potat::test]
async fn test_get_inclusion_states() {
    client_init();
    let res = quorum::get_inclusion_states()
        .transactions(&[
            Hash::from_inner_unchecked(
                TryteBuf::try_from_str(TEST_BUNDLE_TX_0)
                    .unwrap()
                    .as_trits()
                    .encode(),
            ),
            Hash::from_inner_unchecked(
                TryteBuf::try_from_str(TEST_BUNDLE_TX_1)
                    .unwrap()
                    .as_trits()
                    .encode(),
            ),
        ])
        .send()
        .await
        .unwrap();

    assert!(!res.states.is_empty());
}

#[smol_potat::test]
async fn test_get_latest_inclusion() {
    client_init();
    let _ = quorum::get_latest_inclusion(&[
        Hash::from_inner_unchecked(
            TryteBuf::try_from_str(TEST_BUNDLE_TX_0)
                .unwrap()
                .as_trits()
                .encode(),
        ),
        Hash::from_inner_unchecked(
            TryteBuf::try_from_str(TEST_BUNDLE_TX_1)
                .unwrap()
                .as_trits()
                .encode(),
        ),
    ])
    .await;
}
#[smol_potat::test]
async fn test_were_addresses_spent_from() {
    client_init();
    let res = quorum::were_addresses_spent_from(&[Address::from_inner_unchecked(
        TryteBuf::try_from_str(TEST_ADDRESS_0)
            .unwrap()
            .as_trits()
            .encode(),
    )])
    .await
    .unwrap();

    assert_eq!(res.states[0], false);
}
