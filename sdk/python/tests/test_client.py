"""Basic tests for the Chitin Python SDK.

Tests client instantiation, type creation, and ChitinMemory interface.
Phase 1: Tests run without a live daemon connection.
"""

import pytest
from datetime import datetime, timezone


class TestChitinClient:
    """Tests for ChitinClient instantiation and basic operations."""

    def test_client_instantiation_defaults(self):
        """Client should instantiate with default host and port."""
        from chitin.client import ChitinClient

        client = ChitinClient()
        assert client.host == "localhost"
        assert client.port == 50051
        assert client._connected is False

    def test_client_instantiation_custom(self):
        """Client should accept custom host and port."""
        from chitin.client import ChitinClient

        client = ChitinClient(host="10.0.0.1", port=9090)
        assert client.host == "10.0.0.1"
        assert client.port == 9090

    def test_client_context_manager(self):
        """Client should work as a context manager."""
        from chitin.client import ChitinClient

        with ChitinClient() as client:
            assert client._connected is True
        assert client._connected is False

    def test_client_submit_polyp_placeholder(self):
        """submit_polyp should return a placeholder UUID in Phase 1."""
        from chitin.client import ChitinClient

        with ChitinClient() as client:
            polyp_id = client.submit_polyp("Test knowledge text")
            assert isinstance(polyp_id, str)
            assert len(polyp_id) > 0

    def test_client_search_placeholder(self):
        """search should return an empty list in Phase 1."""
        from chitin.client import ChitinClient

        with ChitinClient() as client:
            results = client.search("test query")
            assert isinstance(results, list)
            assert len(results) == 0

    def test_client_get_node_info(self):
        """get_node_info should return a dict with expected keys."""
        from chitin.client import ChitinClient

        with ChitinClient() as client:
            info = client.get_node_info()
            assert isinstance(info, dict)
            assert "node_type" in info
            assert "version" in info
            assert info["connected"] is True


class TestTypes:
    """Tests for Chitin SDK type creation and validation."""

    def test_polyp_state_enum(self):
        """PolypState enum should have all lifecycle states."""
        from chitin.types import PolypState

        assert PolypState.DRAFT == "Draft"
        assert PolypState.SOFT == "Soft"
        assert PolypState.UNDER_REVIEW == "UnderReview"
        assert PolypState.APPROVED == "Approved"
        assert PolypState.HARDENED == "Hardened"
        assert PolypState.REJECTED == "Rejected"
        assert PolypState.MOLTED == "Molted"

    def test_payload_creation(self):
        """Payload should be creatable with minimal args."""
        from chitin.types import Payload

        payload = Payload(content="Hello world")
        assert payload.content == "Hello world"
        assert payload.content_type == "text/plain"
        assert payload.language is None

    def test_embedding_model_id(self):
        """EmbeddingModelId should hold model identification data."""
        from chitin.types import EmbeddingModelId

        model = EmbeddingModelId(
            provider="bge",
            name="bge-small-en-v1.5",
            weights_hash="abc123",
            dimensions=384,
        )
        assert model.provider == "bge"
        assert model.dimensions == 384

    def test_vector_embedding(self):
        """VectorEmbedding should store float values and model reference."""
        from chitin.types import VectorEmbedding, EmbeddingModelId

        model = EmbeddingModelId(
            provider="bge",
            name="bge-small-en-v1.5",
            weights_hash="abc123",
            dimensions=3,
        )
        vec = VectorEmbedding(
            values=[0.1, 0.2, 0.3],
            model_id=model,
        )
        assert len(vec.values) == 3
        assert vec.quantization == "float32"

    def test_polyp_scores_weighted(self):
        """PolypScores.weighted_score should compute correctly."""
        from chitin.types import PolypScores

        scores = PolypScores(
            zk_validity=1.0,
            semantic_quality=0.8,
            novelty=0.6,
            source_credibility=0.7,
            embedding_quality=0.9,
        )
        expected = (
            1.0 * 0.30
            + 0.8 * 0.25
            + 0.6 * 0.15
            + 0.7 * 0.15
            + 0.9 * 0.15
        )
        assert abs(scores.weighted_score() - expected) < 1e-10

    def test_search_result_creation(self):
        """SearchResult should be creatable with required fields."""
        from chitin.types import SearchResult, Payload

        result = SearchResult(
            polyp_id="test-uuid",
            payload=Payload(content="Some text"),
            similarity=0.95,
        )
        assert result.polyp_id == "test-uuid"
        assert result.similarity == 0.95
        assert result.state == "Hardened"

    def test_provenance_creation(self):
        """Provenance should hold creator and source information."""
        from chitin.types import (
            Provenance,
            SourceAttribution,
            ProcessingPipeline,
            PipelineStep,
        )

        prov = Provenance(
            creator_hotkey="aabbccdd",
            creator_did="did:chitin:0xaabbccdd",
            source=SourceAttribution(
                source_url="https://example.com",
                title="Example",
                accessed_at=datetime.now(timezone.utc),
            ),
            pipeline=ProcessingPipeline(
                steps=[
                    PipelineStep(name="chunk", version="1.0", params={}),
                    PipelineStep(name="embed", version="1.0", params={"model": "bge"}),
                ],
                duration_ms=150,
            ),
        )
        assert prov.creator_did.startswith("did:chitin:")
        assert len(prov.pipeline.steps) == 2


class TestChitinMemory:
    """Tests for the ChitinMemory high-level agent interface."""

    def test_memory_instantiation(self):
        """ChitinMemory should instantiate and connect."""
        from chitin.agent import ChitinMemory

        memory = ChitinMemory()
        assert memory._client._connected is True
        memory.close()

    def test_memory_remember(self):
        """remember should return a polyp ID string."""
        from chitin.agent import ChitinMemory

        memory = ChitinMemory()
        polyp_id = memory.remember("The Earth orbits the Sun")
        assert isinstance(polyp_id, str)
        assert len(polyp_id) > 0
        memory.close()

    def test_memory_recall(self):
        """recall should return a list of result dicts."""
        from chitin.agent import ChitinMemory

        memory = ChitinMemory()
        results = memory.recall("planets in solar system")
        assert isinstance(results, list)
        memory.close()

    def test_memory_forget(self):
        """forget should return False in Phase 1 (not implemented)."""
        from chitin.agent import ChitinMemory

        memory = ChitinMemory()
        result = memory.forget("some-uuid")
        assert result is False
        memory.close()
