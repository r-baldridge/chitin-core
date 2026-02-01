"""Python dataclasses mirroring Rust types from chitin-core.

All types use pydantic BaseModel for validation, serialization,
and compatibility with modern Python tooling.
"""

from __future__ import annotations

from datetime import datetime
from enum import Enum
from typing import Any, Optional

from pydantic import BaseModel, Field


class PolypState(str, Enum):
    """Lifecycle states of a Polyp."""
    DRAFT = "Draft"
    SOFT = "Soft"
    UNDER_REVIEW = "UnderReview"
    APPROVED = "Approved"
    HARDENED = "Hardened"
    REJECTED = "Rejected"
    MOLTED = "Molted"


class EmbeddingModelId(BaseModel):
    """Identifies a specific embedding model version."""
    provider: str = Field(description="Model family (e.g., 'openai', 'bge', 'nomic')")
    name: str = Field(description="Model name (e.g., 'text-embedding-3-small')")
    weights_hash: str = Field(description="SHA-256 hash of model weights (hex)")
    dimensions: int = Field(description="Output dimensionality")


class VectorEmbedding(BaseModel):
    """A vector embedding with full model provenance."""
    values: list[float] = Field(description="The raw float vector")
    model_id: EmbeddingModelId = Field(description="Which model produced this vector")
    quantization: str = Field(default="float32", description="Quantization applied")
    normalization: str = Field(default="l2", description="Normalization applied")


class Payload(BaseModel):
    """The human-readable knowledge content."""
    content: str = Field(description="The raw text, code snippet, or structured data")
    content_type: str = Field(default="text/plain", description="MIME type of the content")
    language: Optional[str] = Field(default=None, description="Language code (e.g., 'en', 'es')")


class ZkProof(BaseModel):
    """ZK proof attesting to correct embedding generation."""
    proof_type: str = Field(description="Proof system identifier: 'SP1Groth16', 'Risc0Stark', etc.")
    proof_value: str = Field(description="Hex-encoded proof bytes")
    vk_hash: str = Field(description="The verification key hash (identifies the circuit)")
    text_hash: str = Field(description="SHA-256 hash of the source text (hex)")
    vector_hash: str = Field(description="SHA-256 hash of the resulting vector bytes (hex)")
    model_id: str = Field(description="Embedding model identifier")
    created_at: datetime = Field(description="Timestamp of proof generation")


class SourceAttribution(BaseModel):
    """Attribution to the original source."""
    source_cid: Optional[str] = Field(default=None, description="IPFS CID of the original source document")
    source_url: Optional[str] = Field(default=None, description="URL of the original source")
    title: Optional[str] = Field(default=None, description="Human-readable title or description")
    license: Optional[str] = Field(default=None, description="License under which the source is available")
    accessed_at: datetime = Field(description="Timestamp when the source was accessed")


class PipelineStep(BaseModel):
    """A single step in the processing pipeline."""
    name: str
    version: str
    params: dict[str, Any] = Field(default_factory=dict)


class ProcessingPipeline(BaseModel):
    """Describes the processing pipeline that produced the Polyp."""
    steps: list[PipelineStep] = Field(default_factory=list)
    duration_ms: int = Field(default=0, description="Total wall-clock time (milliseconds)")


class Provenance(BaseModel):
    """Full provenance chain for a Polyp."""
    creator_hotkey: str = Field(description="Hex-encoded hotkey of the creator node")
    creator_did: str = Field(description="DID of the creator node")
    source: SourceAttribution
    pipeline: ProcessingPipeline


class NodeIdentity(BaseModel):
    """Identity of a node on the Chitin network."""
    coldkey: str = Field(description="Coldkey public key (hex)")
    hotkey: str = Field(description="Hotkey public key (hex)")
    did: str = Field(description="DID derived from coldkey")
    node_type: str = Field(description="'Coral', 'Tide', or 'Hybrid'")


class PolypScores(BaseModel):
    """Multi-dimensional quality scores for a Polyp. Each dimension is 0.0 to 1.0."""
    zk_validity: float = Field(ge=0.0, le=1.0, description="Did the ZK proof verify? Binary: 0.0 or 1.0")
    semantic_quality: float = Field(ge=0.0, le=1.0, description="Semantic quality: coherence, informativeness")
    novelty: float = Field(ge=0.0, le=1.0, description="How much new information does this Polyp add?")
    source_credibility: float = Field(ge=0.0, le=1.0, description="Reputation of creator + source quality")
    embedding_quality: float = Field(ge=0.0, le=1.0, description="Cosine similarity vs reference embedding")

    # Default dimension weights for computing final score
    DEFAULT_WEIGHTS: list[float] = [0.30, 0.25, 0.15, 0.15, 0.15]

    def weighted_score(self) -> float:
        """Compute weighted final score."""
        vals = [
            self.zk_validity,
            self.semantic_quality,
            self.novelty,
            self.source_credibility,
            self.embedding_quality,
        ]
        return sum(v * w for v, w in zip(vals, self.DEFAULT_WEIGHTS))


class PolypSubject(BaseModel):
    """The subject of a Polyp: payload (human-readable) + vector (machine-readable)."""
    payload: Payload
    vector: VectorEmbedding
    provenance: Provenance


class Polyp(BaseModel):
    """The atomic unit of knowledge in Reefipedia."""
    id: str = Field(description="Unique identifier (UUID v7)")
    state: PolypState = Field(description="Current lifecycle state")
    subject: PolypSubject = Field(description="The knowledge content: text + embedding + provenance")
    proof: ZkProof = Field(description="ZK proof attesting Vector = Model(Text)")
    consensus: Optional[dict[str, Any]] = Field(default=None, description="Consensus metadata (populated after validation)")
    hardening: Optional[dict[str, Any]] = Field(default=None, description="Hardening lineage (populated after hardening)")
    created_at: datetime = Field(description="Creation timestamp")
    updated_at: datetime = Field(description="Last state transition timestamp")


class SearchResult(BaseModel):
    """A single result from a semantic search query."""
    polyp_id: str = Field(description="Polyp UUID")
    cid: Optional[str] = Field(default=None, description="IPFS CID (if hardened)")
    payload: Payload = Field(description="The text content")
    similarity: float = Field(description="Cosine similarity to query")
    trust_score: float = Field(default=0.0, description="Composite trust score")
    creator_did: str = Field(default="", description="DID of the creator node")
    state: str = Field(default="Hardened", description="Polyp lifecycle state")
    hardened_epoch: Optional[int] = Field(default=None, description="Epoch when hardened")
