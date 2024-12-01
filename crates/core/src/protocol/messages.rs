use serde::{Deserialize, Serialize};
use super::types::{TransactionDigest, SignedTransaction, TransactionCertificate};
use super::errors::ProtocolResult;
use crate::core::{ObjectID, SequenceNumber};

/// 网络消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    /// 共识相关消息
    Consensus(ConsensusMessage),
    /// 请求消息
    Request(RequestMessage),
    /// 响应消息
    Response(ResponseMessage),
}

/// 共识消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsensusMessage {
    /// 提议
    Proposal {
        round: u64,
        transactions: Vec<TransactionDigest>,
    },
    /// 投票
    Vote {
        round: u64,
        proposal: TransactionDigest,
        signature: Vec<u8>,
    },
    /// 超时
    Timeout {
        round: u64,
        signature: Vec<u8>,
    },
}

/// 请求消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequestMessage {
    /// 提交交易
    SubmitTransaction(SignedTransaction),
    /// 提交证书
    SubmitCertificate(TransactionCertificate),
    /// 查询交易信息
    TransactionInfo(TransactionInfoRequest),
    /// 同步对象
    SyncObject {
        id: ObjectID,
        version: Option<SequenceNumber>,
    },
}

/// 响应消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseMessage {
    /// 交易信息响应
    TransactionInfo(TransactionInfoResponse),
    /// 对象同步响应
    ObjectSync {
        id: ObjectID,
        version: SequenceNumber,
        data: Vec<u8>,
    },
    /// 错误响应
    Error {
        code: u32,
        message: String,
    },
}

/// 交易信息请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionInfoRequest {
    pub digest: TransactionDigest,
    pub request_type: TransactionInfoRequestType,
}

/// 交易信息请求类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionInfoRequestType {
    /// 只返回状态
    Status,
    /// 返回完整信息
    Full,
}

/// 交易信息响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionInfoResponse {
    pub digest: TransactionDigest,
    pub status: TransactionStatus,
    pub certificate: Option<TransactionCertificate>,
    pub effects: Option<TransactionEffects>,
}

/// 交易状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionStatus {
    /// 待处理
    Pending,
    /// 已处理
    Processed {
        success: bool,
        error_message: Option<String>,
    },
    /// 已确认
    Confirmed,
}

/// 交易效果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionEffects {
    /// 创建的对象
    pub created: Vec<ObjectID>,
    /// 修改的对象
    pub modified: Vec<ObjectID>,
    /// 删除的对象
    pub deleted: Vec<ObjectID>,
    /// Gas 使用情况
    pub gas_used: u64,
    /// 状态变更
    pub status: ExecutionStatus,
}

/// 执行状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionStatus {
    /// 成功
    Success,
    /// 失败
    Failure {
        error: String,
    },
}

impl NetworkMessage {
    /// 处理网络消息
    pub async fn handle(self, context: &mut Context) -> ProtocolResult<ResponseMessage> {
        match self {
            NetworkMessage::Consensus(msg) => {
                context.handle_consensus_message(msg).await
            }
            NetworkMessage::Request(req) => {
                context.handle_request_message(req).await
            }
            NetworkMessage::Response(resp) => {
                context.handle_response_message(resp).await
            }
        }
    }
}

/// 消息处理上下文
pub struct Context {
    // 添加必要的字段
    consensus: ConsensusHandle,
    mempool: MempoolHandle,
    storage: StorageHandle,
}

impl Context {
    async fn handle_consensus_message(
        &mut self,
        message: ConsensusMessage,
    ) -> ProtocolResult<ResponseMessage> {
        // 实现共识消息处理逻辑
        todo!()
    }

    async fn handle_request_message(
        &mut self,
        request: RequestMessage,
    ) -> ProtocolResult<ResponseMessage> {
        // 实现请求消息处理逻辑
        todo!()
    }

    async fn handle_response_message(
        &mut self,
        response: ResponseMessage,
    ) -> ProtocolResult<ResponseMessage> {
        // 实现响应消息处理逻辑
        todo!()
    }
}