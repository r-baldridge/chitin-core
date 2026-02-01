"""gRPC client for the Chitin Protocol daemon.

Provides a Python interface to the chitin-daemon's gRPC API.
Phase 1: All methods are stubs with placeholder returns.
"""

from __future__ import annotations

from typing import Optional

from chitin.types import Polyp, SearchResult


class ChitinClient:
    """Client for interacting with a chitin-daemon node via gRPC.

    Usage:
        with ChitinClient(host="localhost", port=50051) as client:
            polyp_id = client.submit_polyp("Some knowledge text")
            results = client.search("query about knowledge")

    Phase 1: All methods return placeholder values. Real gRPC
    integration will be added when the daemon is running.
    """

    def __init__(self, host: str = "localhost", port: int = 50051) -> None:
        """Initialize the Chitin client.

        Args:
            host: Hostname or IP of the chitin-daemon.
            port: gRPC port of the chitin-daemon.
        """
        self.host = host
        self.port = port
        self._channel = None
        self._connected = False

    def __enter__(self) -> "ChitinClient":
        """Enter context manager — connect to daemon."""
        self.connect()
        return self

    def __exit__(self, exc_type, exc_val, exc_tb) -> None:
        """Exit context manager — disconnect from daemon."""
        self.close()

    def connect(self) -> None:
        """Establish gRPC connection to the daemon.

        Phase 1: No-op placeholder.
        """
        # Phase 2: self._channel = grpc.insecure_channel(f"{self.host}:{self.port}")
        self._connected = True

    def close(self) -> None:
        """Close the gRPC connection.

        Phase 1: No-op placeholder.
        """
        # Phase 2: if self._channel: self._channel.close()
        self._connected = False
        self._channel = None

    def submit_polyp(
        self,
        text: str,
        content_type: str = "text/plain",
        model_id: Optional[str] = None,
    ) -> str:
        """Submit a new Polyp to the network.

        The daemon will:
        1. Generate an embedding using the specified model.
        2. Create a ZK proof (Phase 3) or skip (Phase 1-2).
        3. Assemble the Polyp with provenance metadata.
        4. Broadcast to the network.

        Args:
            text: The knowledge text to embed and submit.
            content_type: MIME type of the content.
            model_id: Embedding model to use (None = daemon default).

        Returns:
            The UUID of the created Polyp.
        """
        # Phase 1: Placeholder — returns a mock UUID
        # Phase 2: Call SubmitPolyp gRPC endpoint
        return "00000000-0000-0000-0000-000000000000"

    def get_polyp(self, polyp_id: str) -> Polyp:
        """Retrieve a Polyp by its UUID.

        Args:
            polyp_id: The UUID of the Polyp to retrieve.

        Returns:
            The Polyp object with all metadata.

        Raises:
            NotImplementedError: Phase 1 placeholder.
        """
        # Phase 2: Call GetPolyp gRPC endpoint
        raise NotImplementedError("get_polyp not yet implemented — Phase 1 placeholder")

    def search(
        self,
        query: str,
        top_k: int = 10,
        model_id: Optional[str] = None,
    ) -> list[SearchResult]:
        """Perform semantic search over the Reef.

        Args:
            query: Natural language query text.
            top_k: Maximum number of results to return.
            model_id: Embedding model space to search (None = default).

        Returns:
            List of SearchResult objects ordered by similarity.
        """
        # Phase 1: Placeholder — returns empty results
        # Phase 2: Call SemanticSearch gRPC endpoint
        return []

    def get_node_info(self) -> dict:
        """Get information about the connected node.

        Returns:
            Dictionary with node type, version, uptime, capabilities.
        """
        # Phase 1: Placeholder
        # Phase 2: Call GetNodeInfo gRPC endpoint
        return {
            "node_type": "hybrid",
            "version": "0.1.0",
            "uptime_seconds": 0,
            "connected": self._connected,
            "host": self.host,
            "port": self.port,
        }
