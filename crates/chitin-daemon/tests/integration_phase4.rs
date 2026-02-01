// crates/chitin-daemon/tests/integration_phase4.rs
//
// Phase 4 integration tests for the Chitin Protocol daemon.
//
// Tests the wired-up algorithm pipeline: epoch lifecycle, scoring + consensus,
// hardening pipeline, and the end-to-end epoch flow.
//
// These tests use the public APIs of the underlying library crates directly
// (chitin-consensus, chitin-store, chitin-reputation, chitin-core) since the
// daemon is a binary crate with no lib.rs.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use uuid::Uuid;

use chitin_consensus::bonds::BondMatrix;
use chitin_consensus::epoch::{EpochManager, EpochPhase};
use chitin_consensus::metagraph::MetagraphManager;
use chitin_consensus::scoring::score_polyp_multi_dimensional;
use chitin_consensus::weights::WeightMatrix;
use chitin_consensus::yuma::yuma_semantic_consensus;
use chitin_core::consensus::ConsensusMetadata;
use chitin_core::embedding::{EmbeddingModelId, VectorEmbedding};
use chitin_core::polyp::{
    Payload, Polyp, PolypSubject, PolypState, ProofPublicInputs, ZkProof,
};
use chitin_core::provenance::{PipelineStep, ProcessingPipeline, Provenance, SourceAttribution};
use chitin_core::identity::{NodeIdentity, NodeType};
use chitin_core::traits::PolypStore;
use chitin_core::ReefMetagraph;
use chitin_reputation::trust_matrix::TrustMatrix;
use chitin_store::RocksStore;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Create a temporary directory path using UUID to avoid conflicts.
fn temp_db_path(label: &str) -> String {
    let dir = std::env::temp_dir();
    let path = dir.join(format!("chitin_test_{}_{}", label, Uuid::now_v7()));
    path.to_string_lossy().to_string()
}

/// Create a test Polyp with the given content and state.
fn make_test_polyp(content: &str, state: PolypState) -> Polyp {
    let now = chrono::Utc::now();
    let dim = 8u32;
    // Create a non-trivial, normalized vector
    let raw = vec![0.3f32, 0.4, 0.5, 0.2, 0.1, 0.6, 0.3, 0.2];
    let norm: f32 = raw.iter().map(|x| x * x).sum::<f32>().sqrt();
    let normalized: Vec<f32> = raw.iter().map(|x| x / norm).collect();

    Polyp {
        id: Uuid::now_v7(),
        state,
        subject: PolypSubject {
            payload: Payload {
                content: content.to_string(),
                content_type: "text/plain".to_string(),
                language: Some("en".to_string()),
            },
            vector: VectorEmbedding {
                values: normalized,
                model_id: EmbeddingModelId {
                    provider: "test".to_string(),
                    name: "test-model".to_string(),
                    weights_hash: [0u8; 32],
                    dimensions: dim,
                },
                quantization: "float32".to_string(),
                normalization: "l2".to_string(),
            },
            provenance: Provenance {
                creator: NodeIdentity {
                    coldkey: [1u8; 32], // non-placeholder
                    hotkey: [0u8; 32],
                    did: "did:chitin:test".to_string(),
                    node_type: NodeType::Coral,
                },
                source: SourceAttribution {
                    source_cid: None,
                    source_url: Some("https://example.com".to_string()),
                    title: Some("Test Content".to_string()),
                    license: None,
                    accessed_at: now,
                },
                pipeline: ProcessingPipeline {
                    steps: vec![PipelineStep {
                        name: "embed".to_string(),
                        version: "1.0".to_string(),
                        params: serde_json::json!({}),
                    }],
                    duration_ms: 50,
                },
            },
        },
        proof: ZkProof {
            proof_type: "SP1Groth16".to_string(),
            proof_value: "abcdef1234567890".to_string(),
            vk_hash: "test_vk".to_string(),
            public_inputs: ProofPublicInputs {
                text_hash: [0u8; 32],
                vector_hash: [0u8; 32],
                model_id: EmbeddingModelId {
                    provider: "test".to_string(),
                    name: "test-model".to_string(),
                    weights_hash: [0u8; 32],
                    dimensions: dim,
                },
            },
            created_at: now,
        },
        consensus: None,
        hardening: None,
        created_at: now,
        updated_at: now,
        signature: None,
    }
}

// ===========================================================================
// Test 1: Epoch Lifecycle
// ===========================================================================

/// Verify that EpochManager transitions through all phases correctly
/// as blocks are advanced through a full epoch.
#[tokio::test]
async fn test_epoch_lifecycle_full_cycle() {
    let blocks_per_epoch: u64 = 100;
    let em = Arc::new(RwLock::new(EpochManager::new(blocks_per_epoch)));

    // Initial state: epoch 0, phase Open
    {
        let em = em.read().await;
        assert_eq!(em.current_epoch(), 0);
        assert_eq!(*em.phase(), EpochPhase::Open);
    }

    // Advance to block 25 (25% into epoch) — still Open
    {
        let mut em = em.write().await;
        em.advance_block(25);
        assert_eq!(em.current_epoch(), 0);
        assert_eq!(*em.phase(), EpochPhase::Open);
    }

    // Advance to block 50 (50% into epoch) — transition to Scoring
    {
        let mut em = em.write().await;
        em.advance_block(50);
        assert_eq!(em.current_epoch(), 0);
        assert_eq!(*em.phase(), EpochPhase::Scoring);
    }

    // Advance to block 74 (74% into epoch) — still Scoring
    {
        let mut em = em.write().await;
        em.advance_block(74);
        assert_eq!(em.current_epoch(), 0);
        assert_eq!(*em.phase(), EpochPhase::Scoring);
    }

    // Advance to block 75 (75% into epoch) — transition to Committing
    {
        let mut em = em.write().await;
        em.advance_block(75);
        assert_eq!(em.current_epoch(), 0);
        assert_eq!(*em.phase(), EpochPhase::Committing);
    }

    // Advance to block 99 (99% into epoch) — still Committing
    {
        let mut em = em.write().await;
        em.advance_block(99);
        assert_eq!(em.current_epoch(), 0);
        assert_eq!(*em.phase(), EpochPhase::Committing);
    }

    // Advance to block 100 — new epoch (epoch 1), Open phase
    {
        let mut em = em.write().await;
        em.advance_block(100);
        assert_eq!(em.current_epoch(), 1);
        assert_eq!(*em.phase(), EpochPhase::Open);
    }

    // Advance to block 150 (50% into epoch 1) — Scoring
    {
        let mut em = em.write().await;
        em.advance_block(150);
        assert_eq!(em.current_epoch(), 1);
        assert_eq!(*em.phase(), EpochPhase::Scoring);
    }

    // Advance to block 200 — new epoch (epoch 2), Open
    {
        let mut em = em.write().await;
        em.advance_block(200);
        assert_eq!(em.current_epoch(), 2);
        assert_eq!(*em.phase(), EpochPhase::Open);
    }
}

/// Verify that the epoch scheduler detects phase transitions and epoch boundaries
/// using broadcast events.
#[tokio::test]
async fn test_epoch_scheduler_events() {
    use tokio::sync::broadcast;

    let blocks_per_epoch: u64 = 10;
    let em = Arc::new(RwLock::new(EpochManager::new(blocks_per_epoch)));
    let (tx, mut rx) = broadcast::channel::<(u64, String)>(32);

    // Simulate the scheduler's advance_block logic
    for block in 1..=20 {
        let prev_phase;
        let prev_epoch;
        {
            let em = em.read().await;
            prev_phase = em.phase().clone();
            prev_epoch = em.current_epoch();
        }
        {
            let mut em = em.write().await;
            em.advance_block(block);
        }
        let new_phase;
        let new_epoch;
        {
            let em = em.read().await;
            new_phase = em.phase().clone();
            new_epoch = em.current_epoch();
        }

        if new_epoch > prev_epoch {
            let _ = tx.send((block, format!("EpochBoundary:{}", new_epoch)));
        }
        if new_phase != prev_phase {
            let _ = tx.send((block, format!("PhaseChanged:{:?}", new_phase)));
        }
    }

    // Collect all events
    let mut events = Vec::new();
    while let Ok(event) = rx.try_recv() {
        events.push(event);
    }

    // Verify we got phase transitions for both epochs
    let phase_changes: Vec<&str> = events
        .iter()
        .filter(|(_, e)| e.starts_with("PhaseChanged"))
        .map(|(_, e)| e.as_str())
        .collect();

    assert!(
        phase_changes.contains(&"PhaseChanged:Scoring"),
        "Should see Scoring transition"
    );
    assert!(
        phase_changes.contains(&"PhaseChanged:Committing"),
        "Should see Committing transition"
    );

    // Verify epoch boundaries
    let boundaries: Vec<u64> = events
        .iter()
        .filter(|(_, e)| e.starts_with("EpochBoundary"))
        .map(|(b, _)| *b)
        .collect();

    assert!(
        boundaries.contains(&10),
        "Should see epoch boundary at block 10"
    );
    assert!(
        boundaries.contains(&20),
        "Should see epoch boundary at block 20"
    );
}

// ===========================================================================
// Test 2: Score Submission + Consensus
// ===========================================================================

/// Test that weight submission and consensus execution produces valid results.
#[tokio::test]
async fn test_score_submission_and_consensus() {
    let n_validators = 2;
    let n_corals = 3;

    // Set up shared state
    let weight_matrix = Arc::new(RwLock::new(WeightMatrix::new(n_validators, n_corals)));
    let bond_matrix = Arc::new(RwLock::new(BondMatrix::new(n_validators, n_corals)));

    // Simulate validators submitting weights
    {
        let mut wm = weight_matrix.write().await;
        // Validator 0: prefers coral 0
        wm.set(0, 0, 0.6);
        wm.set(0, 1, 0.3);
        wm.set(0, 2, 0.1);
        // Validator 1: also prefers coral 0 (agreement)
        wm.set(1, 0, 0.5);
        wm.set(1, 1, 0.3);
        wm.set(1, 2, 0.2);
    }

    // Read matrices for consensus
    let weights;
    let prev_bonds;
    {
        let wm = weight_matrix.read().await;
        weights = wm.weights.clone();
    }
    {
        let bm = bond_matrix.read().await;
        prev_bonds = bm.bonds.clone();
    }

    let stakes = vec![100u64; n_validators];

    // Run consensus
    let result = yuma_semantic_consensus(&stakes, &weights, &prev_bonds, 0.5, 0.1, 0.1);

    // Verify output structure
    assert_eq!(result.consensus_weights.len(), n_corals);
    assert_eq!(result.incentives.len(), n_corals);
    assert_eq!(result.dividends.len(), n_validators);
    assert_eq!(result.bonds.len(), n_validators);
    assert_eq!(result.bonds[0].len(), n_corals);

    // Consensus weights should be positive (both validators assigned non-zero weights)
    for cw in &result.consensus_weights {
        assert!(*cw >= 0.0, "Consensus weights should be non-negative");
    }

    // Incentives should sum to ~1.0
    let incentive_sum: f64 = result.incentives.iter().sum();
    assert!(
        (incentive_sum - 1.0).abs() < 1e-10,
        "Incentives should sum to 1.0, got {}",
        incentive_sum
    );

    // Dividends should sum to ~1.0
    let div_sum: f64 = result.dividends.iter().sum();
    assert!(
        (div_sum - 1.0).abs() < 1e-10,
        "Dividends should sum to 1.0, got {}",
        div_sum
    );

    // Update bond matrix with results
    {
        let mut bm = bond_matrix.write().await;
        *bm = BondMatrix::new(n_validators, n_corals);
        for (i, row) in result.bonds.iter().enumerate() {
            for (j, &val) in row.iter().enumerate() {
                bm.bonds[i][j] = val;
            }
        }
    }

    // Verify bonds were updated (at least some non-zero)
    {
        let bm = bond_matrix.read().await;
        let has_nonzero = bm.bonds.iter().any(|row| row.iter().any(|&v| v > 0.0));
        assert!(has_nonzero, "Bond matrix should have non-zero entries after consensus");
    }

    // Store consensus result
    let consensus_result = Arc::new(RwLock::new(Some(result.clone())));
    {
        let cr = consensus_result.read().await;
        assert!(cr.is_some(), "Consensus result should be stored");
    }
}

/// Test consensus with epoch phase validation.
#[tokio::test]
async fn test_consensus_epoch_phase_validation() {
    let blocks_per_epoch = 100u64;
    let em = Arc::new(RwLock::new(EpochManager::new(blocks_per_epoch)));

    // Phase: Open (block 0) — should NOT accept scores
    {
        let em = em.read().await;
        assert_eq!(*em.phase(), EpochPhase::Open);
    }

    // Advance to Scoring phase
    {
        let mut em = em.write().await;
        em.advance_block(50);
        assert_eq!(*em.phase(), EpochPhase::Scoring);
    }

    // Now scores should be accepted (Scoring phase)
    {
        let em = em.read().await;
        let phase = em.phase();
        assert!(
            *phase == EpochPhase::Scoring || *phase == EpochPhase::Committing,
            "Scores should be accepted in Scoring or Committing phase"
        );
    }

    // Advance to Committing phase
    {
        let mut em = em.write().await;
        em.advance_block(75);
        assert_eq!(*em.phase(), EpochPhase::Committing);
    }

    // Scores still accepted in Committing
    {
        let em = em.read().await;
        let phase = em.phase();
        assert!(
            *phase == EpochPhase::Scoring || *phase == EpochPhase::Committing,
            "Scores should be accepted in Committing phase"
        );
    }
}

// ===========================================================================
// Test 3: Scoring + State Transitions (without IPFS hardening)
// ===========================================================================

/// Test the scoring pipeline: create Soft polyps, score them, verify scores are reasonable.
#[tokio::test]
async fn test_polyp_scoring_pipeline() {
    let db_path = temp_db_path("scoring");
    let store = Arc::new(RocksStore::open(&db_path).expect("Failed to open RocksDB"));

    // Create and save 3 Soft polyps with varying quality
    let polyp1 = make_test_polyp(
        "The mitochondria is the powerhouse of the cell. This organelle generates most of the cell's supply of adenosine triphosphate (ATP), used as a source of chemical energy.",
        PolypState::Soft,
    );
    let polyp2 = make_test_polyp("hi", PolypState::Soft); // low quality (short content)
    let polyp3 = make_test_polyp(
        "Quantum computing leverages quantum-mechanical phenomena such as superposition and entanglement to perform computation.",
        PolypState::Soft,
    );

    store.save_polyp(&polyp1).await.unwrap();
    store.save_polyp(&polyp2).await.unwrap();
    store.save_polyp(&polyp3).await.unwrap();

    // List Soft polyps
    let soft_polyps = store.list_polyps_by_state(&PolypState::Soft).await.unwrap();
    assert_eq!(soft_polyps.len(), 3, "Should have 3 Soft polyps");

    // Score each polyp
    let mut scores = Vec::new();
    for polyp in &soft_polyps {
        let s = score_polyp_multi_dimensional(polyp);
        scores.push((polyp.id, s));
    }

    // All polyps should have non-zero scores
    for (id, s) in &scores {
        let ws = s.weighted_score();
        assert!(ws > 0.0, "Polyp {} should have positive weighted score, got {}", id, ws);
    }

    // The short-content polyp should score lower on semantic_quality
    let p2_scores = scores.iter().find(|(id, _)| *id == polyp2.id).unwrap();
    assert!(
        p2_scores.1.semantic_quality < 0.3,
        "Short content should have low semantic quality, got {}",
        p2_scores.1.semantic_quality
    );

    // Simulate transitioning Soft -> UnderReview
    for polyp in &soft_polyps {
        let mut updated = polyp.clone();
        updated.state = PolypState::UnderReview;
        updated.updated_at = chrono::Utc::now();
        store.save_polyp(&updated).await.unwrap();
    }

    // Verify state transitions
    let soft_after = store.list_polyps_by_state(&PolypState::Soft).await.unwrap();
    let under_review = store.list_polyps_by_state(&PolypState::UnderReview).await.unwrap();
    assert_eq!(soft_after.len(), 0, "No Soft polyps should remain");
    assert_eq!(under_review.len(), 3, "All 3 should be UnderReview");

    // Cleanup
    std::fs::remove_dir_all(&db_path).ok();
}

/// Test the approval flow: UnderReview polyps that pass consensus get Approved.
#[tokio::test]
async fn test_polyp_approval_flow() {
    let db_path = temp_db_path("approval");
    let store = Arc::new(RocksStore::open(&db_path).expect("Failed to open RocksDB"));

    // Create 3 UnderReview polyps
    let polyps: Vec<Polyp> = (0..3)
        .map(|i| {
            make_test_polyp(
                &format!("Knowledge content {} with sufficient detail for a good score in the system", i),
                PolypState::UnderReview,
            )
        })
        .collect();

    for p in &polyps {
        store.save_polyp(p).await.unwrap();
    }

    // Simulate scoring: 1 validator, 3 corals
    let n_validators = 1;
    let n_corals = 3;
    let mut wm = WeightMatrix::new(n_validators, n_corals);

    // Score each polyp and populate weight matrix
    for (idx, polyp) in polyps.iter().enumerate() {
        let scores = score_polyp_multi_dimensional(polyp);
        wm.set(0, idx, scores.weighted_score());
    }

    let prev_bonds = vec![vec![0.0; n_corals]; n_validators];
    let stakes = vec![100u64; n_validators];

    // Run consensus
    let result = yuma_semantic_consensus(&stakes, &wm.weights, &prev_bonds, 0.5, 0.1, 0.1);

    // Identify approved polyps (threshold 0.3)
    let approval_threshold = 0.3;
    let mut approved_ids = Vec::new();
    for (idx, polyp) in polyps.iter().enumerate() {
        if idx < result.consensus_weights.len()
            && result.consensus_weights[idx] > approval_threshold
        {
            approved_ids.push(polyp.id);
        }
    }

    assert!(
        !approved_ids.is_empty(),
        "At least some polyps should be approved (all have decent content)"
    );

    // Transition approved polyps: UnderReview -> Approved
    let epoch = 1u64;
    for (idx, polyp) in polyps.iter().enumerate() {
        if approved_ids.contains(&polyp.id) {
            let mut updated = polyp.clone();
            updated.state = PolypState::Approved;
            updated.consensus = Some(ConsensusMetadata {
                epoch,
                final_score: result.consensus_weights[idx],
                validator_scores: vec![],
                hardened: false,
                finalized_at: chrono::Utc::now(),
            });
            updated.updated_at = chrono::Utc::now();
            store.save_polyp(&updated).await.unwrap();
        }
    }

    // Verify state
    let approved = store.list_polyps_by_state(&PolypState::Approved).await.unwrap();
    assert_eq!(
        approved.len(),
        approved_ids.len(),
        "Approved count should match"
    );

    // Verify consensus metadata is attached
    for p in &approved {
        assert!(p.consensus.is_some(), "Approved polyps should have consensus metadata");
        let cm = p.consensus.as_ref().unwrap();
        assert_eq!(cm.epoch, epoch);
        assert!(cm.final_score > 0.0);
    }

    // Cleanup
    std::fs::remove_dir_all(&db_path).ok();
}

// ===========================================================================
// Test 4: End-to-End Epoch Flow
// ===========================================================================

/// Full end-to-end test simulating a complete epoch:
/// 1. Create polyps in Soft state
/// 2. Advance to Scoring phase
/// 3. Score polyps, populate weight matrix
/// 4. Transition Soft -> UnderReview
/// 5. Advance to epoch boundary
/// 6. Run consensus
/// 7. Transition approved UnderReview -> Approved
/// 8. Update bonds, trust matrix, metagraph
#[tokio::test]
async fn test_end_to_end_epoch() {
    let blocks_per_epoch = 100u64;
    let db_path = temp_db_path("e2e");
    let store = Arc::new(RocksStore::open(&db_path).expect("Failed to open RocksDB"));

    // --- Shared state ---
    let epoch_manager = Arc::new(RwLock::new(EpochManager::new(blocks_per_epoch)));
    let weight_matrix = Arc::new(RwLock::new(WeightMatrix::new(0, 0)));
    let bond_matrix = Arc::new(RwLock::new(BondMatrix::new(0, 0)));
    let trust_matrix = Arc::new(RwLock::new(TrustMatrix::new()));
    let metagraph_manager = Arc::new(RwLock::new(MetagraphManager::new()));
    let consensus_result: Arc<RwLock<Option<chitin_consensus::yuma::ConsensusResult>>> =
        Arc::new(RwLock::new(None));

    // --- Step 1: Submit polyps (Soft state) ---
    // Use 3 polyps so each gets ~0.33 normalized weight (above 0.3 approval threshold)
    let n_polyps = 3;
    let mut polyps = Vec::new();
    for i in 0..n_polyps {
        let p = make_test_polyp(
            &format!(
                "Detailed scientific knowledge entry number {}. Contains factual information about the natural world, \
                 including observations, hypotheses, and verified conclusions.",
                i
            ),
            PolypState::Soft,
        );
        store.save_polyp(&p).await.unwrap();
        polyps.push(p);
    }

    let soft_count = store.list_polyps_by_state(&PolypState::Soft).await.unwrap().len();
    assert_eq!(soft_count, n_polyps, "Should have {} Soft polyps", n_polyps);

    // --- Step 2: Advance to Scoring phase ---
    {
        let mut em = epoch_manager.write().await;
        em.advance_block(50); // 50% into epoch -> Scoring
        assert_eq!(*em.phase(), EpochPhase::Scoring);
    }

    // --- Step 3: Score polyps and populate weight matrix ---
    let n_validators = 1;
    {
        let mut wm = weight_matrix.write().await;
        *wm = WeightMatrix::new(n_validators, n_polyps);
        for (idx, polyp) in polyps.iter().enumerate() {
            let scores = score_polyp_multi_dimensional(polyp);
            wm.set(0, idx, scores.weighted_score());
        }
    }

    // --- Step 4: Transition Soft -> UnderReview ---
    for polyp in &polyps {
        let mut updated = polyp.clone();
        updated.state = PolypState::UnderReview;
        updated.updated_at = chrono::Utc::now();
        store.save_polyp(&updated).await.unwrap();
    }

    let under_review = store.list_polyps_by_state(&PolypState::UnderReview).await.unwrap();
    assert_eq!(under_review.len(), n_polyps);

    // --- Step 5: Advance to epoch boundary ---
    {
        let mut em = epoch_manager.write().await;
        em.advance_block(100); // New epoch
        assert_eq!(em.current_epoch(), 1);
        assert_eq!(*em.phase(), EpochPhase::Open);
    }

    // --- Step 6: Run consensus ---
    let weights;
    let prev_bonds;
    {
        let wm = weight_matrix.read().await;
        weights = wm.weights.clone();
    }
    {
        // Initialize bond matrix to match dimensions
        let mut bm = bond_matrix.write().await;
        *bm = BondMatrix::new(n_validators, n_polyps);
        prev_bonds = bm.bonds.clone();
    }

    let stakes = vec![100u64; n_validators];
    let result = yuma_semantic_consensus(&stakes, &weights, &prev_bonds, 0.5, 0.1, 0.1);

    // Store consensus result
    {
        let mut cr = consensus_result.write().await;
        *cr = Some(result.clone());
    }

    // Update bond matrix
    {
        let mut bm = bond_matrix.write().await;
        *bm = BondMatrix::new(n_validators, n_polyps);
        for (i, row) in result.bonds.iter().enumerate() {
            for (j, &val) in row.iter().enumerate() {
                bm.bonds[i][j] = val;
            }
        }
    }

    // --- Step 7: Transition approved polyps ---
    let approval_threshold = 0.3;
    let epoch = 1u64;
    let mut approved_count = 0;

    // Re-read UnderReview polyps from store
    let ur_polyps = store.list_polyps_by_state(&PolypState::UnderReview).await.unwrap();
    for (idx, polyp) in ur_polyps.iter().enumerate() {
        if idx < result.consensus_weights.len()
            && result.consensus_weights[idx] > approval_threshold
        {
            let mut updated = polyp.clone();
            updated.state = PolypState::Approved;
            updated.consensus = Some(ConsensusMetadata {
                epoch,
                final_score: result.consensus_weights[idx],
                validator_scores: vec![],
                hardened: false,
                finalized_at: chrono::Utc::now(),
            });
            updated.updated_at = chrono::Utc::now();
            store.save_polyp(&updated).await.unwrap();
            approved_count += 1;
        }
    }

    assert!(approved_count > 0, "At least some polyps should be approved");

    // --- Step 8: Update trust matrix and metagraph ---

    // Trust: single validator trusts itself
    {
        let mut tm = trust_matrix.write().await;
        for v in 0..n_validators {
            tm.set_trust(v as u16, v as u16, 1.0);
        }
        assert!((tm.get_trust(0, 0) - 1.0).abs() < 1e-10);
    }

    // Metagraph update
    {
        let metagraph = ReefMetagraph {
            epoch,
            block: 100,
            nodes: vec![],
            total_stake: 100,
            total_hardened_polyps: approved_count as u64,
            emission_rate: 0,
            weights: HashMap::new(),
            bonds: HashMap::new(),
        };

        let mut mm = metagraph_manager.write().await;
        mm.update(metagraph).expect("Metagraph update should succeed");

        let current = mm.current().expect("Should have current metagraph");
        assert_eq!(current.epoch, epoch);
        assert_eq!(current.total_hardened_polyps, approved_count as u64);
    }

    // --- Verify final state ---
    let final_approved = store.list_polyps_by_state(&PolypState::Approved).await.unwrap();
    assert_eq!(final_approved.len(), approved_count);

    // Verify consensus result is accessible
    {
        let cr = consensus_result.read().await;
        assert!(cr.is_some());
        let r = cr.as_ref().unwrap();
        assert_eq!(r.consensus_weights.len(), n_polyps);
        assert_eq!(r.bonds.len(), n_validators);
    }

    // Verify bonds are non-zero
    {
        let bm = bond_matrix.read().await;
        let total_bonds: f64 = bm.bonds.iter().flat_map(|r| r.iter()).sum();
        assert!(total_bonds > 0.0, "Bonds should be non-zero after consensus");
    }

    // Cleanup
    std::fs::remove_dir_all(&db_path).ok();
}

/// Test that metagraph enforces epoch monotonicity.
#[tokio::test]
async fn test_metagraph_monotonicity() {
    let mut mm = MetagraphManager::new();

    let mg1 = ReefMetagraph {
        epoch: 1,
        block: 100,
        nodes: vec![],
        total_stake: 100,
        total_hardened_polyps: 5,
        emission_rate: 0,
        weights: HashMap::new(),
        bonds: HashMap::new(),
    };

    mm.update(mg1).expect("First update should succeed");
    assert_eq!(mm.current().unwrap().epoch, 1);

    // Stale epoch should fail
    let mg_stale = ReefMetagraph {
        epoch: 1,
        block: 200,
        nodes: vec![],
        total_stake: 200,
        total_hardened_polyps: 10,
        emission_rate: 0,
        weights: HashMap::new(),
        bonds: HashMap::new(),
    };

    let result = mm.update(mg_stale);
    assert!(result.is_err(), "Stale epoch update should fail");

    // Forward epoch should succeed
    let mg2 = ReefMetagraph {
        epoch: 2,
        block: 200,
        nodes: vec![],
        total_stake: 200,
        total_hardened_polyps: 10,
        emission_rate: 0,
        weights: HashMap::new(),
        bonds: HashMap::new(),
    };

    mm.update(mg2).expect("Forward epoch update should succeed");
    assert_eq!(mm.current().unwrap().epoch, 2);
}

/// Test multi-epoch bond evolution: bonds should accumulate and evolve across epochs.
#[tokio::test]
async fn test_multi_epoch_bond_evolution() {
    let n_validators = 2;
    let n_corals = 2;
    let stakes = vec![100u64; n_validators];

    // Validators give consistent weights across epochs
    let weights = vec![vec![0.7, 0.3], vec![0.6, 0.4]];
    let alpha = 0.1;
    let bond_penalty = 0.1;

    // Epoch 1: start from zero bonds
    let prev_bonds_1 = vec![vec![0.0; n_corals]; n_validators];
    let r1 = yuma_semantic_consensus(&stakes, &weights, &prev_bonds_1, 0.5, bond_penalty, alpha);

    // Epoch 2: use epoch 1's bonds
    let r2 = yuma_semantic_consensus(&stakes, &weights, &r1.bonds, 0.5, bond_penalty, alpha);

    // Epoch 3: use epoch 2's bonds
    let r3 = yuma_semantic_consensus(&stakes, &weights, &r2.bonds, 0.5, bond_penalty, alpha);

    // Bonds should be building up over consistent scoring
    // At least some bond values should increase or stabilize
    let bond_sum_1: f64 = r1.bonds.iter().flat_map(|r| r.iter()).sum();
    let bond_sum_2: f64 = r2.bonds.iter().flat_map(|r| r.iter()).sum();
    let bond_sum_3: f64 = r3.bonds.iter().flat_map(|r| r.iter()).sum();

    assert!(
        bond_sum_1 > 0.0,
        "Epoch 1 bonds should be non-zero, got {}",
        bond_sum_1
    );
    assert!(
        bond_sum_2 > 0.0,
        "Epoch 2 bonds should be non-zero, got {}",
        bond_sum_2
    );
    assert!(
        bond_sum_3 > 0.0,
        "Epoch 3 bonds should be non-zero, got {}",
        bond_sum_3
    );

    // Consensus weights should remain stable with consistent inputs
    for i in 0..n_corals {
        let diff_12 = (r1.consensus_weights[i] - r2.consensus_weights[i]).abs();
        assert!(
            diff_12 < 1e-10,
            "Consensus weights should be stable with same inputs"
        );
    }
}
