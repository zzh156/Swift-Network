use rand::{Rng, rngs::OsRng}; // 使用 OsRng 和 Rng trait
use ed25519_dalek::{Keypair, PublicKey, Signature, Signer, Verifier};
use bip39::{Mnemonic, Language}; // 更新 Seed 的引入
use sha2::{Sha256, Digest}; // 导入 SHA256 哈希
use std::str;

pub struct Crypto {}

impl Crypto {
    // 生成助记词
    pub fn generate_mnemonic() -> String {
        let mut csprng = OsRng; // 创建操作系统随机数生成器
        let entropy: [u8; 16] = csprng.gen(); // 使用 Rng trait 直接生成 128 位（16 字节）的数组
        let mnemonic = Mnemonic::from_entropy(&entropy).expect("助记词生成失败！");
        mnemonic.to_string() // 将 Mnemonic 转为 String
    }

    // 从助记词生成密钥对
    pub fn keypair_from_mnemonic(mnemonic_str: &str) -> (PublicKey, Keypair) {
        let mnemonic = Mnemonic::parse_in_normalized(Language::English, mnemonic_str)
            .expect("无效的助记词！");
        
        // 手动生成种子（模拟 to_seed 方法）
        let mut hasher = Sha256::new();
        hasher.update(mnemonic_str.as_bytes());
        let seed = hasher.finalize();
        
        // 从种子生成密钥对
        let keypair = Keypair::from_bytes(&seed[..32]).expect("密钥生成失败！");
        (keypair.public, keypair)
    }

    // 使用 CSPRNG 生成密钥对
    pub fn generate_keypair() -> (PublicKey, Keypair) {
        let mut csprng = OsRng; // 创建操作系统随机数生成器
        let keypair = Keypair::generate(&mut csprng); // 生成密钥对
        (keypair.public, keypair)
    }

    // 签名消息
    pub fn sign_message(message: &str, keypair: &Keypair) -> Signature {
        keypair.sign(message.as_bytes())
    }

    // 验证签名
    pub fn verify_signature(message: &str, signature: &Signature, public_key: &PublicKey) -> bool {
        public_key.verify(message.as_bytes(), signature).is_ok()
    }
}
