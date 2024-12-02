export interface Wallet {
    address: string;
    publicKey: string;
    privateKey?: string;
    mnemonic?: string;
    name: string;
  }
  
  export interface Transaction {
    from: string;
    to: string;
    amount: string;
    gasPrice?: string;
    gasLimit?: string;
    data?: string;
    nonce?: number;
  }
  
  export interface NetworkConfig {
    name: string;
    rpcUrl: string;
    chainId: string;
    symbol: string;
    explorer: string;
  }
  
  export interface WalletState {
    currentWallet: Wallet | null;
    wallets: Wallet[];
    balance: string;
    network: NetworkConfig;
  }