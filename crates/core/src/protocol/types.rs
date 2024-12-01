use serde::{Deserialize, Serialize};
use crate::core::{ObjectID, SequenceNumber};
use crate::crypto::{PublicKey, Signature};

/// 交易摘要
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct TransactionDigest([u8; 32]);

/// 交易数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionData {
    /// 交易发送者
    pub sender: PublicKey,
    /// 交易类型
    pub kind: TransactionKind,
    /// Gas 预算
    pub gas_budget: u64,
    /// Gas 价格
    pub gas_price: u64,
    /// 交易过期时间
    pub expiration: u64,
}

/// 交易类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionKind {
    /// 转移对象
    TransferObject {
        object_id: ObjectID,
        recipient: PublicKey,
        version: SequenceNumber,
    },
    /// 发布包
    Publish {
        modules: Vec<Vec<u8>>,
    },
    /// 调用函数
    MoveCall {
        package: ObjectID,
        module: String,
        function: String,
        type_arguments: Vec<TypeTag>,
        arguments: Vec<CallArg>,
    },
}

/// 已签名的交易
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedTransaction {
    pub data: TransactionData,
    pub signature: Signature,
}

/// 交易证书
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionCertificate {
    pub transaction: SignedTransaction,
    pub authority_signatures: Vec<(PublicKey, Signature)>,
}

/// 类型标签
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TypeTag {
    Bool,
    U8,
    U64,
    U128,
    Address,
    Vector(Box<TypeTag>),
    Struct(StructTag),
}

/// 结构体标签
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructTag {
    pub address: String,
    pub module: String,
    pub name: String,
    pub type_args: Vec<TypeTag>,
}

/// 调用参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CallArg {
    Pure(Vec<u8>),
    Object(ObjectID),
    ObjVec(Vec<ObjectID>),
}