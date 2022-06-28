package node_api_core;

import org.iota.Client;
import org.iota.types.ClientConfig;
import org.iota.types.ClientException;
import org.iota.types.MilestonePayload;
import org.iota.types.ids.MilestoneId;
import org.iota.types.responses.TreasuryResponse;

public class GetMilestoneById {

    private static final String DEFAULT_TESTNET_NODE_URL = "http://localhost:14265";
    private static ClientConfig config = new ClientConfig("{ \"nodes\": [\"" + DEFAULT_TESTNET_NODE_URL + "\" ], \"nodeSyncEnabled\": false}");

    public static void main(String[] args) throws ClientException {
        // Build the client.
        Client client = new Client(config);

        // Set up a milestone id for this example.
        MilestoneId milestoneId = ExampleUtils.setUpMilestoneId(client);

        // Get the milestone.
        MilestonePayload milestone = client.getMilestoneById(milestoneId);

        // Print the milestone.
        System.out.println(milestone);
    }

}
