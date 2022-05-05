// Copyright 2021-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0
import type { FeatureBlock } from '../featureBlocks';
import type { TypeBase } from '../typeBase';
import type { CommonOutput } from './commonOutput';

/**
 * The global type for the alias output.
 */
export const ALIAS_OUTPUT_TYPE = 4;

export interface AliasOutput extends TypeBase<4>, CommonOutput {
    /**
     * Amount of IOTA tokens held by the output.
     */
    amount: string;
    /**
     * Unique identifier of the alias, which is the BLAKE2b-160 hash of the Output ID that created it.
     */
    aliasId: string;
    /**
     * A counter that must increase by 1 every time the alias is state transitioned.
     */
    stateIndex: number;
    /**
     * Metadata that can only be changed by the state controller.
     */
    stateMetadata: string;
    /**
     * A counter that denotes the number of foundries created by this alias account.
     */
    foundryCounter: number;
    /**
     * Immutable blocks contained by the output.
     */
    immutableFeatureBlocks: FeatureBlock[];
}
