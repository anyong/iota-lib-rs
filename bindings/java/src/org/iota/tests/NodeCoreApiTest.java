package org.iota.tests;

import org.iota.main.types.*;
import org.iota.main.types.responses.node_core_api.NodeInfoResponse;
import org.iota.main.types.responses.node_core_api.TreasuryResponse;
import org.iota.main.types.responses.node_core_api.UtxoChangesResponse;
import org.junit.jupiter.api.Test;

import java.util.Map;

public class NodeCoreApiTest extends ApiTest {

    @Test
    public void testGetHealth() throws ClientException {
        boolean health = client.getHealth(DEFAULT_DEVNET_NODE_URL);
        System.out.println(health);
    }

    @Test
    public void testGetNodeInfo() throws ClientException {
        NodeInfoResponse r = client.getNodeInfo();
        System.out.println(r.getNodeInfo());
    }

    @Test
    public void testGetTips() throws ClientException {
        for (BlockId tip : client.getTips())
            System.out.println(tip);
    }

    @Test
    public void testPostBlock() throws ClientException {
        BlockId blockId = client.postBlock(setUpTaggedDataBlock());
        System.out.println(blockId);
    }

    @Test
    public void testGetBlock() throws ClientException {
        Block block = client.getBlock(client.postBlock(setUpTaggedDataBlock()));
        System.out.println(block);
    }

    @Test
    public void testGetBlockRaw() throws ClientException {
        byte[] blockBytes = client.getBlockRaw(client.postBlock(setUpTaggedDataBlock()));
    }

    @Test
    public void testGetBlockMetadata() throws ClientException {
        System.out.println(client.getBlockMetadata(client.postBlock(setUpTaggedDataBlock())));
    }

    @Test
    public void testGetOutput() throws ClientException {
        Map.Entry<Output, OutputMetadata> r = client.getOutputWithMetadata(setupOutputId());
        System.out.println(r.getKey());
        System.out.println(r.getValue());
    }

    @Test
    public void testGetOutputMetadata() throws ClientException {
        OutputMetadata r = client.getOutputMetadata(setupOutputId());
        System.out.println(r);
    }

    @Test
    public void testGetReceiptsMigratedAt() throws ClientException {
        Receipt[] receipts = client.getReceiptsMigratedAt(client.getNodeInfo().getNodeInfo().get("status").getAsJsonObject().get("latestMilestone").getAsJsonObject().get("index").getAsInt());
        for (Receipt r : receipts)
            System.out.println(r);
    }

    @Test
    public void testGetReceipts() throws ClientException {
        Receipt[] receipts = client.getReceipts();
        for (Receipt r : receipts)
            System.out.println(r);
    }

    @Test
    public void testGetTreasury() throws ClientException {
        TreasuryResponse r = client.getTreasury();
        System.out.println(r);
    }

    @Test
    public void testGetIncludedBlock() throws ClientException {
        System.out.println(client.getIncludedBlock(setUpTransactionId()));
    }

    @Test
    public void testGetMilestoneById() throws ClientException {
        MilestoneId milestoneId = new MilestoneId(client.getNodeInfo().getNodeInfo().get("status").getAsJsonObject().get("latestMilestone").getAsJsonObject().get("milestoneId").getAsString());
        Milestone r = client.getMilestoneById(milestoneId);
        System.out.println(r);
    }

    @Test
    public void testGetMilestoneByIndex() throws ClientException {
        Milestone r = client.getMilestoneByIndex(client.getNodeInfo().getNodeInfo().get("status").getAsJsonObject().get("latestMilestone").getAsJsonObject().get("index").getAsInt());
        System.out.println(r);
    }

    @Test
    public void testGetMilestoneByIdRaw() throws ClientException {
        MilestoneId milestoneId = new MilestoneId(client.getNodeInfo().getNodeInfo().get("status").getAsJsonObject().get("latestMilestone").getAsJsonObject().get("milestoneId").getAsString());
        byte[] milestoneBytes = client.getMilestoneByIdRaw(milestoneId);
    }

    @Test
    public void testGetMilestoneByIndexRaw() throws ClientException {
        byte[] milestoneBytes = client.getMilestoneByIndexRaw(client.getNodeInfo().getNodeInfo().get("status").getAsJsonObject().get("latestMilestone").getAsJsonObject().get("index").getAsInt());
    }

    @Test
    public void testGetUtxoChangesId() throws ClientException {
        MilestoneId milestoneId = new MilestoneId(client.getNodeInfo().getNodeInfo().get("status").getAsJsonObject().get("latestMilestone").getAsJsonObject().get("milestoneId").getAsString());
        UtxoChangesResponse r = client.getUtxoChangesById(milestoneId);
        for (OutputId consumed : r.getConsumedOutputs())
            System.out.println(consumed);
        for (OutputId created : r.getCreatedOutputs())
            System.out.println(created);
    }

    @Test
    public void testGetUtxoChangesIndex() throws ClientException {
        UtxoChangesResponse r = client.getUtxoChangesByIndex(client.getNodeInfo().getNodeInfo().get("status").getAsJsonObject().get("latestMilestone").getAsJsonObject().get("index").getAsInt());
        for (OutputId consumed : r.getConsumedOutputs())
            System.out.println(consumed);
        for (OutputId created : r.getCreatedOutputs())
            System.out.println(created);
    }

    @Test
    public void testGetPeers() throws ClientException {
        for (Peer peer : client.getPeers())
            System.out.println(peer);
    }

}
