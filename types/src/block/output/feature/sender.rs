// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use derive_more::From;

use crate::block::address::Address;

/// Identifies the validated sender of an output.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, From, packable::Packable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SenderFeature(Address);

impl SenderFeature {
    /// The [`Feature`](crate::block::output::Feature) kind of a [`SenderFeature`].
    pub const KIND: u8 = 0;

    /// Creates a new [`SenderFeature`].
    #[inline(always)]
    pub const fn new(address: Address) -> Self {
        Self(address)
    }

    /// Returns the sender [`Address`].
    #[inline(always)]
    pub const fn address(&self) -> &Address {
        &self.0
    }
}

#[cfg(feature = "dto")]
#[allow(missing_docs)]
pub mod dto {
    use serde::{Deserialize, Serialize};

    use crate::block::address::dto::AddressDto;

    #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
    pub struct SenderFeatureDto {
        #[serde(rename = "type")]
        pub kind: u8,
        pub address: AddressDto,
    }
}
