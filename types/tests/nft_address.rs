// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::str::FromStr;

use iota_types::block::{
    address::{dto::NftAddressDto, NftAddress},
    output::NftId,
    DtoError,
};

const NFT_ID: &str = "0xb0c800965d7511f5fb4406274d4e607f87d5c5970bc05e896f841a700e86eafb";
const NFT_ID_INVALID: &str = "0xb0c800965d7511f5fb4406274d4e607f87d5c5970bc05e896f841a700e86e";

#[test]
fn kind() {
    assert_eq!(NftAddress::KIND, 16);
}

#[test]
fn length() {
    assert_eq!(NftAddress::LENGTH, 32);
}

#[test]
fn new_nft_id() {
    let nft_id = NftId::from_str(NFT_ID).unwrap();
    let nft_address = NftAddress::new(nft_id);

    assert_eq!(nft_address.nft_id(), &nft_id);
}

#[test]
fn new_into_nft_id() {
    let nft_id = NftId::from_str(NFT_ID).unwrap();
    let nft_address = NftAddress::new(nft_id);

    assert_eq!(nft_address.into_nft_id(), nft_id);
}

#[test]
fn from_str_to_str() {
    let nft_address = NftAddress::from_str(NFT_ID).unwrap();

    assert_eq!(nft_address.to_string(), NFT_ID);
}

#[test]
fn debug() {
    let nft_address = NftAddress::from_str(NFT_ID).unwrap();

    assert_eq!(
        format!("{nft_address:?}"),
        "NftAddress(0xb0c800965d7511f5fb4406274d4e607f87d5c5970bc05e896f841a700e86eafb)"
    );
}

#[test]
fn dto_fields() {
    let nft_address = NftAddress::from_str(NFT_ID).unwrap();
    let dto = NftAddressDto::from(&nft_address);

    assert_eq!(dto.kind, NftAddress::KIND);
    assert_eq!(dto.nft_id, NFT_ID.to_string());
}

#[test]
fn address_dto_roundtrip() {
    let nft_address = NftAddress::from_str(NFT_ID).unwrap();
    let dto = NftAddressDto::from(&nft_address);

    assert_eq!(NftAddress::try_from(&dto).unwrap(), nft_address);
}

#[test]
fn dto_invalid_nft_id() {
    let dto = NftAddressDto {
        kind: NftAddress::KIND,
        nft_id: NFT_ID_INVALID.to_string(),
    };

    assert!(matches!(
        NftAddress::try_from(&dto),
        Err(DtoError::InvalidField("nftId"))
    ));
}
