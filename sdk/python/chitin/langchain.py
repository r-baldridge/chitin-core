"""LangChain VectorStore adapter for the Chitin Protocol.

Provides a LangChain-compatible VectorStore interface that uses
the Chitin Reef as the backing store. This allows AI agents built
with LangChain to use Reefipedia as their knowledge base.

Phase 1: Placeholder implementations. The langchain-core dependency
is optional and may not be installed.
"""

from __future__ import annotations

from typing import Any, Iterable, Optional

from chitin.client import ChitinClient
from chitin.types import SearchResult


class ChitinVectorStore:
    """LangChain VectorStore adapter for the Chitin Reef.

    In Phase 2+, this class will inherit from langchain_core.vectorstores.VectorStore.
    For Phase 1, it implements the same interface as a standalone class.

    Usage:
        store = ChitinVectorStore()
        store.add_texts(["fact one", "fact two"])
        results = store.similarity_search("query", k=5)
    """

    def __init__(
        self,
        host: str = "localhost",
        port: int = 50051,
        model_id: Optional[str] = None,
        collection_name: Optional[str] = None,
    ) -> None:
        """Initialize the ChitinVectorStore.

        Args:
            host: Hostname of the chitin-daemon.
            port: gRPC port of the chitin-daemon.
            model_id: Embedding model to use (None = daemon default).
            collection_name: Optional Reef Zone name to scope searches.
        """
        self._client = ChitinClient(host=host, port=port)
        self._client.connect()
        self._model_id = model_id
        self._collection_name = collection_name

    def add_texts(
        self,
        texts: Iterable[str],
        metadatas: Optional[list[dict[str, Any]]] = None,
        **kwargs: Any,
    ) -> list[str]:
        """Add texts to the Reef as Polyps.

        Args:
            texts: Iterable of text strings to add.
            metadatas: Optional list of metadata dicts (one per text).
            **kwargs: Additional keyword arguments.

        Returns:
            List of Polyp UUIDs for the added texts.
        """
        ids = []
        for i, text in enumerate(texts):
            polyp_id = self._client.submit_polyp(
                text=text,
                content_type="text/plain",
                model_id=self._model_id,
            )
            ids.append(polyp_id)
        return ids

    def similarity_search(
        self,
        query: str,
        k: int = 4,
        **kwargs: Any,
    ) -> list[dict[str, Any]]:
        """Search the Reef for similar documents.

        Args:
            query: Natural language query string.
            k: Number of results to return.
            **kwargs: Additional keyword arguments.

        Returns:
            List of document-like dictionaries with page_content and metadata.
            In Phase 2+, these will be LangChain Document objects.
        """
        results: list[SearchResult] = self._client.search(
            query=query,
            top_k=k,
            model_id=self._model_id,
        )
        return [
            {
                "page_content": r.payload.content,
                "metadata": {
                    "polyp_id": r.polyp_id,
                    "cid": r.cid,
                    "similarity": r.similarity,
                    "trust_score": r.trust_score,
                    "creator_did": r.creator_did,
                    "state": r.state,
                    "hardened_epoch": r.hardened_epoch,
                },
            }
            for r in results
        ]

    def similarity_search_with_score(
        self,
        query: str,
        k: int = 4,
        **kwargs: Any,
    ) -> list[tuple[dict[str, Any], float]]:
        """Search the Reef and return results with similarity scores.

        Args:
            query: Natural language query string.
            k: Number of results to return.
            **kwargs: Additional keyword arguments.

        Returns:
            List of (document_dict, similarity_score) tuples.
        """
        results: list[SearchResult] = self._client.search(
            query=query,
            top_k=k,
            model_id=self._model_id,
        )
        return [
            (
                {
                    "page_content": r.payload.content,
                    "metadata": {
                        "polyp_id": r.polyp_id,
                        "cid": r.cid,
                        "trust_score": r.trust_score,
                        "creator_did": r.creator_did,
                        "state": r.state,
                        "hardened_epoch": r.hardened_epoch,
                    },
                },
                r.similarity,
            )
            for r in results
        ]

    def close(self) -> None:
        """Close the underlying client connection."""
        self._client.close()
