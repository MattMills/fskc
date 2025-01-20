use fskc::{
    Result,
    triplet::{
        TimingVerificationNode, RFState, EntropyFeed, QuantumSeedExchange,
        ExchangeMode, StateProof, FeedType, FeedProof, SNRMeasurement, 
        SignalState, LatencyMeasurement, IVParameter,
    },
    entropy::EntropySource,
};
use std::time::{SystemTime, UNIX_EPOCH};

struct DummyEntropySource;

impl EntropySource for DummyEntropySource {
    fn fill_bytes(&mut self, dest: &mut [u8]) -> Result<()> {
        for (i, byte) in dest.iter_mut().enumerate() {
            *byte = i as u8;
        }
        Ok(())
    }

    fn description(&self) -> &str {
        "Dummy Entropy Source"
    }
}

fn main() -> Result<()> {
    // Create timing verification node
    let mut timing_node = TimingVerificationNode::new(1);  // 1ns resolution

    println!("Starting triplet verification system...\n");

    // Add latency measurements
    println!("Adding latency measurements...");
    for i in 0..5 {
        let measurement = LatencyMeasurement {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            latency_ns: i * 1000,  // Increasing latency
            confidence: 0.9,
        };
        timing_node.add_measurement(measurement);
        println!("  Added measurement: {}ns (confidence: {:.2})", 
            i * 1000, 0.9);
    }

    // Generate and verify timing proof
    let proof = timing_node.generate_proof();
    let verified = timing_node.verify_proof(&proof);
    println!("\nTiming proof verification: {}\n", if verified { "SUCCESS" } else { "FAILED" });

    // Create RF state
    let mut rf_state = RFState::new();

    // Add signal states
    println!("Adding RF signal states...");
    for i in 0..3 {
        let bssid = format!("AP_{}", i);
        let state = SignalState {
            signal_strength: -50 - (i * 10) as i32,
            frequency: 2400 + (i * 100) as u32,
            last_seen: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        rf_state.add_signal_state(bssid.clone(), state);
        println!("  Added {}: {}dBm @ {}MHz", bssid, -50 - (i * 10), 2400 + (i * 100));
    }

    // Add SNR measurements
    println!("\nAdding SNR measurements...");
    for i in 0..3 {
        let measurement = SNRMeasurement {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            snr_db: 25.0 - (i as f64 * 2.0),
            frequency: 2400 + (i * 100) as u32,
        };
        rf_state.add_snr_measurement(measurement);
        println!("  Added measurement: {:.1}dB @ {}MHz", 
            25.0 - (i as f64 * 2.0), 2400 + (i * 100));
    }

    // Add IV parameters
    println!("\nAdding IV parameters...");
    for i in 0..3 {
        let parameter = IVParameter {
            value: vec![i as u8; 16],
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            bssid: format!("AP_{}", i),
        };
        rf_state.add_iv_parameter(parameter);
        println!("  Added IV for AP_{}", i);
    }

    // Create entropy feeds
    println!("\nCreating entropy feeds...");
    let mut feeds = Vec::new();

    // RF feed
    let rf_feed = EntropyFeed::new(FeedType::RF, Box::new(DummyEntropySource));
    feeds.push(rf_feed);

    // Acoustic feed
    let acoustic_feed = EntropyFeed::new(FeedType::Acoustic, Box::new(DummyEntropySource));
    feeds.push(acoustic_feed);

    // Optical feed
    let optical_feed = EntropyFeed::new(FeedType::Optical, Box::new(DummyEntropySource));
    feeds.push(optical_feed);

    // Add proofs to feeds
    for (i, feed) in feeds.iter_mut().enumerate() {
        let proof = FeedProof {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            data: vec![i as u8; 32],
            verified: true,
        };
        feed.add_proof(proof);
        println!("  Added {} feed (quality: {:.2})", 
            match feed.feed_type {
                FeedType::RF => "RF",
                FeedType::Acoustic => "Acoustic",
                FeedType::Optical => "Optical",
            },
            feed.quality());
    }

    // Create quantum seed exchange
    println!("\nInitializing quantum seed exchange...");
    let exchange = QuantumSeedExchange::new(ExchangeMode::Optical);
    let seed = exchange.generate_seed();
    println!("  Generated seed ({} bytes)", seed.len());
    println!("  Exchange quality: {:.2}", exchange.quality());


    println!("\nTriplet verification system complete!");

    Ok(())
}
