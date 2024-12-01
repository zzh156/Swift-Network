//! Protocol module defines core types and messages for the Sui blockchain.

mod certificate;
mod errors;
mod messages;
mod types;

pub use certificate::{CertificateBuilder, TransactionCertificate};
pub use errors::{ProtocolError, ProtocolResult};
pub use messages::{
    ConsensusMessage, NetworkMessage, RequestMessage, ResponseMessage,
    TransactionInfoRequest, TransactionInfoResponse,
};
pub use types::{
    CallArg, SignedTransaction, StructTag, TransactionData,
    TransactionDigest, TransactionKind, TypeTag,
};

// Protocol constants
pub const PROTOCOL_VERSION: u64 = 1;
pub const MAX_GAS_BUDGET: u64 = 1_000_000;
pub const MAX_TX_SIZE: usize = 128 * 1024;  // 128KB
pub const MAX_DISPLAY_STRING_SIZE: usize = 1024;