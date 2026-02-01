// crates/chitin-store/src/ipfs.rs
//
// IPFS client stub for content-addressed immutable storage.
//
// Phase 1: All methods are `todo!()` stubs. The actual IPFS integration
// will be implemented in Phase 2 using `ipfs-api-backend-hyper` (or equivalent)
// to communicate with a local or remote IPFS/Kubo daemon.
//
// The IPFS client is responsible for:
//   - Pinning hardened Polyps to ensure persistence
//   - Unpinning Polyps that have been superseded (molted)
//   - Retrieving Polyp data by CID
//   - Putting new Polyp data and returning its CID

/// IPFS client for interacting with a Kubo / IPFS daemon.
///
/// In Phase 2, this will be backed by `ipfs-api-backend-hyper` connecting
/// to a local or remote IPFS daemon over HTTP. For now, all methods are
/// unimplemented stubs that will panic if called.
#[derive(Debug, Clone)]
pub struct IpfsClient {
    /// Base URL of the IPFS HTTP API (e.g., "http://127.0.0.1:5001").
    pub base_url: String,
}

impl IpfsClient {
    /// Create a new IPFS client pointing at the given API base URL.
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
        }
    }

    /// Pin a CID to the local IPFS node, ensuring the data is retained.
    ///
    /// Phase 2: POST /api/v0/pin/add?arg={cid}
    pub fn pin(&self, _cid: &str) {
        todo!("IPFS pin: will be implemented with ipfs-api-backend-hyper in Phase 2")
    }

    /// Unpin a CID from the local IPFS node, allowing garbage collection.
    ///
    /// Phase 2: POST /api/v0/pin/rm?arg={cid}
    pub fn unpin(&self, _cid: &str) {
        todo!("IPFS unpin: will be implemented with ipfs-api-backend-hyper in Phase 2")
    }

    /// Retrieve raw bytes for a given CID from the IPFS network.
    ///
    /// Phase 2: POST /api/v0/cat?arg={cid}
    pub fn get_by_cid(&self, _cid: &str) -> Vec<u8> {
        todo!("IPFS get_by_cid: will be implemented with ipfs-api-backend-hyper in Phase 2")
    }

    /// Store raw bytes on IPFS and return the resulting CID.
    ///
    /// Phase 2: POST /api/v0/add (multipart)
    pub fn put(&self, _data: &[u8]) -> String {
        todo!("IPFS put: will be implemented with ipfs-api-backend-hyper in Phase 2")
    }
}
