```json
NodeInfoWrapper{
  node_info: InfoResponse{
    name: "HORNET",
    version: "2.0.0-beta.5",
    status: StatusResponse{
      is_healthy: true,
      latest_milestone: LatestMilestoneResponse{
        index: 749747,
        timestamp: 1661882824,
        milestone_id: "0xcb2d179f9d0a5eea6d0479d9ce01d06cb1dbb1eecd567d74b6ac4a2e46aa5095",
        
      },
      confirmed_milestone: ConfirmedMilestoneResponse{
        index: 749747,
        timestamp: 1661882824,
        milestone_id: "0xcb2d179f9d0a5eea6d0479d9ce01d06cb1dbb1eecd567d74b6ac4a2e46aa5095",
        
      },
      pruning_index: 0,
      
    },
    supported_protocol_versions: [
      2,
      
    ],
    protocol: ProtocolResponse{
      version: 2,
      network_name: "testnet",
      bech32_hrp: "rms",
      min_pow_score: 1500.0,
      rent_structure: RentStructureResponse{
        v_byte_cost: 100,
        v_byte_factor_key: 10,
        v_byte_factor_data: 1,
        
      },
      token_supply: "1450896407249092",
      
    },
    pending_protocol_parameters: [
      
    ],
    base_token: BaseTokenResponse{
      name: "Shimmer",
      ticker_symbol: "SMR",
      unit: "SMR",
      subunit: Some("glow",
      ),
      decimals: 6,
      use_metric_prefix: false,
      
    },
    metrics: MetricsResponse{
      blocks_per_second: 9.0,
      referenced_blocks_per_second: 8.6,
      referenced_rate: 95.55555555555556,
      
    },
    features: [
      
    ],
    
  },
  url: "https://api.testnet.shimmer.network",
  
}
```