package org.iota.main;

import org.iota.main.apis.*;
import org.iota.main.types.*;
import org.iota.main.types.responses.Bech32ToHexResponse;
import org.iota.main.types.responses.GenerateAddressesResponse;
import org.iota.main.types.responses.SuccessResponse;
import org.iota.main.types.responses.node_core_api.*;
import org.iota.main.types.responses.node_indexer_api.OutputIdResponse;
import org.iota.main.types.responses.node_indexer_api.OutputIdsResponse;
import org.iota.main.types.responses.utils.ComputeAliasIdResponse;
import org.iota.main.types.responses.utils.ComputeFoundryIdResponse;
import org.iota.main.types.responses.utils.ComputeNftIdResponse;
import org.iota.main.types.secret.GenerateAddressesOptions;
import org.iota.main.types.secret.GenerateBlockOptions;
import org.iota.main.types.secret.SecretManager;

public class Client {

    private NodeCoreApi nodeCoreApi;
    private NodeIndexerApi nodeIndexerApi;
    private HighLevelApi highLevelApi;
    private UtilsApi utilsApi;
    private MiscellaneousApi miscellaneousApi;

    public Client(ClientConfig config) {
        nodeCoreApi = new NodeCoreApi(config);
        nodeIndexerApi = new NodeIndexerApi(config);
        highLevelApi = new HighLevelApi(config);
        utilsApi = new UtilsApi(config);
        miscellaneousApi = new MiscellaneousApi(config);
    }

    // Node Core APIs

    public HealthResponse getHealth(String nodeUrl) throws ClientException {
        return nodeCoreApi.getHealth(nodeUrl);
    }

    public NodeInfoResponse getNodeInfo() throws ClientException {
        return nodeCoreApi.getNodeInfo();
    }

    public TipsResponse getTips() throws ClientException {
        return nodeCoreApi.getTips();
    }

    public PostBlockResponse postBlock(Block block) throws ClientException {
        return nodeCoreApi.postBlock(block);
    }

    public BlockResponse getBlock(String blockId) throws ClientException {
        return nodeCoreApi.getBlock(blockId);
    }

    public BlockRawResponse getBlockRaw(String blockId) throws ClientException {
        return nodeCoreApi.getBlockRaw(blockId);
    }

    public BlockMetadataResponse getBlockMetadata(String blockId) throws ClientException {
        return nodeCoreApi.getBlockMetadata(blockId);
    }

    public OutputResponse getOutputWithMetadata(String outputId) throws ClientException {
        return nodeCoreApi.getOutputWithMetadata(outputId);
    }

    public OutputMetadataResponse getOutputMetadata(String outputId) throws ClientException {
        return nodeCoreApi.getOutputMetadata(outputId);
    }

    public ReceiptsMigratedAtResponse getReceiptsMigratedAt(int milestoneIndex) throws ClientException {
        return nodeCoreApi.getReceiptsMigratedAt(milestoneIndex);
    }

    public ReceiptsResponse getReceipts() throws ClientException {
        return nodeCoreApi.getReceipts();
    }

    public TreasuryResponse getTreasury() throws ClientException {
        return nodeCoreApi.getTreasury();
    }

    public BlockResponse getIncludedBlock(String transactionId) throws ClientException {
        return nodeCoreApi.getIncludedBlock(transactionId);
    }

    public MilestoneResponse getMilestoneById(String milestoneId) throws ClientException {
        return nodeCoreApi.getMilestoneById(milestoneId);
    }

    public MilestoneResponse getMilestoneByIndex(int milestoneIndex) throws ClientException {
        return nodeCoreApi.getMilestoneByIndex(milestoneIndex);
    }

    public MilestoneRawResponse getMilestoneByIdRaw(String milestoneId) throws ClientException {
        return nodeCoreApi.getMilestoneByIdRaw(milestoneId);
    }

    public MilestoneRawResponse getMilestoneByIndexRaw(int milestoneIndex) throws ClientException {
        return nodeCoreApi.getMilestoneByIndexRaw(milestoneIndex);
    }

    public UtxoChangesResponse getUtxoChangesById(String milestoneId) throws ClientException {
        return nodeCoreApi.getUtxoChangesById(milestoneId);
    }

    public UtxoChangesResponse getUtxoChangesByIndex(int milestoneIndex) throws ClientException {
        return nodeCoreApi.getUtxoChangesByIndex(milestoneIndex);
    }

    public PeersResponse getPeers() throws ClientException {
        return nodeCoreApi.getPeers();
    }

    // Node Indexer APIs

    public OutputIdsResponse getBasicOutputIds(NodeIndexerApi.QueryParams params) throws ClientException {
        return nodeIndexerApi.getBasicOutputIds(params);
    }

    public OutputIdsResponse getAliasOutputIds(NodeIndexerApi.QueryParams params) throws ClientException {
        return nodeIndexerApi.getAliasOutputIds(params);
    }

    public OutputIdsResponse getNftOutputIds(NodeIndexerApi.QueryParams params) throws ClientException {
        return nodeIndexerApi.getNftOutputIds(params);
    }

    public OutputIdsResponse getFoundryOutputIds(NodeIndexerApi.QueryParams params) throws ClientException {
        return nodeIndexerApi.getFoundryOutputIds(params);
    }

    public OutputIdResponse getAliasOutputIdByAliasId(String aliasId) throws ClientException {
        return nodeIndexerApi.getAliasOutputIdByAliasId(aliasId);
    }

    public OutputIdResponse getNftOutputIdByNftId(String nftId) throws ClientException {
        return nodeIndexerApi.getNftOutputIdByNftId(nftId);
    }


    public OutputIdResponse getFoundryOutputIdByFoundryId(String foundryId) throws ClientException {
        return nodeIndexerApi.getFoundryOutputIdByFoundryId(foundryId);
    }

    // High level APIs

    public SuccessResponse getOutputs(String[] outputIds) throws ClientException {
        return highLevelApi.getOutputs(outputIds);
    }

    public SuccessResponse tryGetOutputs(String[] outputIds) throws ClientException {
        return highLevelApi.tryGetOutputs(outputIds);
    }

    public SuccessResponse findMessages(String[] messageIds) throws ClientException {
        return highLevelApi.findMessages(messageIds);
    }

    public SuccessResponse retry(String messageId) throws ClientException {
        return highLevelApi.retry(messageId);
    }

    public SuccessResponse retryUntilIncluded(String messageId, int interval, int maxAttempts) throws ClientException {
        return highLevelApi.retryUntilIncluded(messageId, interval, maxAttempts);
    }

    public SuccessResponse consolidateFunds(SecretManager secretManager, int accountIndex, int addressRange) throws ClientException {
        return highLevelApi.consolidateFunds(secretManager, accountIndex, addressRange);
    }

    public SuccessResponse findInputs(String[] addresses, int amount) throws ClientException {
        return highLevelApi.findInputs(addresses, amount);
    }

    public SuccessResponse findOutputs(String[] outputs, String[] addresses) throws ClientException {
        return highLevelApi.findOutputs(outputs, addresses);
    }

    public SuccessResponse reattach(String messageId) throws ClientException {
        return highLevelApi.reattach(messageId);
    }

    public SuccessResponse reattachUnchecked(String messageId) throws ClientException {
        return highLevelApi.reattachUnchecked(messageId);
    }

    public SuccessResponse promote(String messageId) throws ClientException {
        return highLevelApi.promote(messageId);
    }

    public SuccessResponse promoteUnchecked(String messageId) throws ClientException {
        return highLevelApi.promoteUnchecked(messageId);
    }

    // Utils APIs

    public Bech32ToHexResponse bech32ToHex(String bech32) throws ClientException {
        return utilsApi.bech32ToHex(bech32);
    }

    public SuccessResponse hexToBech32(String hex, String bech32) throws ClientException {
        return utilsApi.hexToBech32(hex, bech32);
    }

    public SuccessResponse hexPublicKeyToBech32Address(String hex, String bech32) throws ClientException {
        return utilsApi.hexPublicKeyToBech32Address(hex, bech32);
    }

    public SuccessResponse parseBech32Address(String address) throws ClientException {
        return utilsApi.parseBech32Address(address);
    }

    public SuccessResponse isAddressValid(String address) throws ClientException {
        return utilsApi.isAddressValid(address);
    }

    public SuccessResponse generateMnemonic() throws ClientException {
        return utilsApi.generateMnemonic();
    }

    public SuccessResponse mnemonicToHexSeed(String mnemonic) throws ClientException {
        return utilsApi.mnemonicToHexSeed(mnemonic);
    }

    public SuccessResponse getBlockId(String block) throws ClientException {
        return utilsApi.getBlockId(block);
    }

    public TransactionIdResponse getTransactionId(BlockPayload payload) throws ClientException {
        return utilsApi.getTransactionId(payload);
    }

    public ComputeAliasIdResponse computeAliasId(String aliasOutputId) throws ClientException {
        return utilsApi.computeAliasId(aliasOutputId);
    }

    public ComputeNftIdResponse computeNftId(String nftOutputId) throws ClientException {
        return utilsApi.computeNftId(nftOutputId);
    }

    public ComputeFoundryIdResponse computeFoundryId(String aliasAddress, int serialNumber, int tokenScheme) throws ClientException {
        return utilsApi.computeFoundryId(aliasAddress, serialNumber, tokenScheme);
    }

    // Miscellaneous APIs

    public GenerateAddressesResponse generateAddresses(SecretManager secretManager, GenerateAddressesOptions generateAddressesOptions) throws ClientException {
        return miscellaneousApi.generateAddresses(secretManager, generateAddressesOptions);
    }

    public BlockResponse generateBlock(SecretManager secretManager, GenerateBlockOptions options) throws ClientException {
        return miscellaneousApi.generateBlock(secretManager, options);
    }


    public SuccessResponse getNode() throws ClientException {
        return miscellaneousApi.getNode();
    }

    public SuccessResponse getNetworkInfo() throws ClientException {
        return miscellaneousApi.getNetworkInfo();
    }

    public SuccessResponse getNetworkId() throws ClientException {
        return miscellaneousApi.getNetworkId();
    }

    public SuccessResponse getBech32Hrp() throws ClientException {
        return miscellaneousApi.getBech32Hrp();
    }

    public SuccessResponse getMinPoWScore() throws ClientException {
        return miscellaneousApi.getMinPoWScore();
    }

    public SuccessResponse getTipsInterval() throws ClientException {
        return miscellaneousApi.getTipsInterval();
    }

    public SuccessResponse getLocalPoW() throws ClientException {
        return miscellaneousApi.getLocalPoW();
    }

    public SuccessResponse getFallbackToLocalPoW() throws ClientException {
        return miscellaneousApi.getFallbackToLocalPoW();
    }

    public SuccessResponse getUsyncedNodes() throws ClientException {
        return miscellaneousApi.getUnsyncedNodes();
    }

    public SuccessResponse prepareTransaction(SecretManager secretManager, GenerateAddressesOptions generateAddressesOptions) throws ClientException {
        return miscellaneousApi.prepareTransaction(secretManager, generateAddressesOptions);
    }

    public SuccessResponse signTransaction(SecretManager secretManager, PreparedTransactionData preparedTransactionData) throws ClientException {
        return miscellaneousApi.signTransaction(secretManager, preparedTransactionData);
    }

    public SuccessResponse storeMnemonic(SecretManager secretManager, String mnemonic) throws ClientException {
        return miscellaneousApi.storeMnemonic(secretManager, mnemonic);
    }

    public BlockResponse submitBlockPayload(BlockPayload payload) throws ClientException {
        return miscellaneousApi.submitBlockPayload(payload);
    }

}

