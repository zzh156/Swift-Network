import { defineStore } from 'pinia';
import { WalletService } from '../services/wallet';
import type { WalletState, Wallet, NetworkConfig } from '../types';

const defaultNetwork: NetworkConfig = {
  name: 'Swift Network',
  rpcUrl: 'https://rpc.swift.network',
  chainId: '1',
  symbol: 'SWIFT',
  explorer: 'https://explorer.swift.network'
};

export const useWalletStore = defineStore('wallet', {
  state: (): WalletState => ({
    currentWallet: null,
    wallets: [],
    balance: '0',
    network: defaultNetwork,
  }),

  actions: {
    async createWallet(password: string) {
      const walletService = new WalletService();
      try {
        const { address, mnemonic } = await walletService.createWallet(password);
        return { success: true, address, mnemonic };
      } catch (error) {
        return { success: false, error: (error as Error).message };
      }
    },

    async importWallet(mnemonic: string, password: string) {
      const walletService = new WalletService();
      try {
        const address = await walletService.importFromMnemonic(mnemonic, password);
        return { success: true, address };
      } catch (error) {
        return { success: false, error: (error as Error).message };
      }
    },

    async loadWallets() {
      // 实现从 chrome.storage 加载钱包列表
    },

    async updateBalance() {
      // 实现余额更新逻辑
    },
  },
});