"""ChitinMemory — high-level agent interface for AI agent integration.

Provides a simple remember/recall/forget API designed for use by
AI agents (LangChain, LlamaIndex, CrewAI, AutoGen, etc.).
"""

from __future__ import annotations

from typing import Any, Optional

from chitin.client import ChitinClient


class ChitinMemory:
    """High-level memory interface for AI agents.

    Wraps ChitinClient with a simplified API:
    - remember(text) -> store knowledge as a Polyp
    - recall(query)  -> semantic search over stored knowledge
    - forget(id)     -> delete a Polyp

    Usage:
        memory = ChitinMemory()
        polyp_id = memory.remember("The speed of light is 299,792,458 m/s")
        results = memory.recall("how fast is light?")
        memory.forget(polyp_id)
    """

    def __init__(
        self,
        host: str = "localhost",
        port: int = 50051,
        default_model: Optional[str] = None,
    ) -> None:
        """Initialize ChitinMemory.

        Args:
            host: Hostname of the chitin-daemon.
            port: gRPC port of the chitin-daemon.
            default_model: Default embedding model ID. None uses daemon default.
        """
        self._client = ChitinClient(host=host, port=port)
        self._client.connect()
        self._default_model = default_model

    def remember(
        self,
        text: str,
        source_url: Optional[str] = None,
        metadata: Optional[dict[str, Any]] = None,
    ) -> str:
        """Store knowledge as a Polyp in the Reef.

        Args:
            text: The knowledge text to store.
            source_url: Optional URL of the original source.
            metadata: Optional additional metadata dictionary.

        Returns:
            The UUID of the created Polyp.
        """
        # Phase 1: Submit via client; source_url and metadata
        # will be attached as provenance in Phase 2
        polyp_id = self._client.submit_polyp(
            text=text,
            content_type="text/plain",
            model_id=self._default_model,
        )
        return polyp_id

    def recall(
        self,
        query: str,
        top_k: int = 5,
    ) -> list[dict[str, Any]]:
        """Search the Reef for relevant knowledge.

        Args:
            query: Natural language query.
            top_k: Maximum number of results.

        Returns:
            List of dictionaries with keys:
            - polyp_id: str
            - text: str
            - similarity: float
            - trust_score: float
            - state: str
        """
        results = self._client.search(
            query=query,
            top_k=top_k,
            model_id=self._default_model,
        )
        return [
            {
                "polyp_id": r.polyp_id,
                "text": r.payload.content,
                "similarity": r.similarity,
                "trust_score": r.trust_score,
                "state": r.state,
            }
            for r in results
        ]

    def forget(self, polyp_id: str) -> bool:
        """Delete a Polyp from the local store.

        Note: Hardened Polyps cannot be deleted from the network
        (they are immutable on IPFS). This only removes the local
        reference and marks the Polyp as forgotten locally.

        Args:
            polyp_id: The UUID of the Polyp to forget.

        Returns:
            True if the Polyp was successfully forgotten, False otherwise.
        """
        # Phase 1: Placeholder — deletion not yet implemented
        # Phase 2: Call delete endpoint on daemon
        return False

    def close(self) -> None:
        """Close the underlying client connection."""
        self._client.close()

    def __del__(self) -> None:
        """Cleanup on garbage collection."""
        try:
            self.close()
        except Exception:
            pass
