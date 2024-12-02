import { Ed25519Keypair } from '@mysten/sui.js';
import { generateMnemonic, mnemonicToSeed } from 'bip39';
import { encrypt, decrypt } from '../utils/crypto';

export class WalletService {
  private static readonly WALLET_STORAGE_KEY = 'swift_wallets';
  private static readonly CURRENT_WALLET_KEY = 'current_wallet';

  async createWallet(password: string): Promise<{
    address: string;
    mnemonic: string;
  }> {
    const mnemonic = generateMnemonic(256); // 24 words
    const seed = await mnemonicToSeed(mnemonic);
    const keypair = Ed25519Keypair.fromSeed(seed);
    const address = keypair.getPublicKey().toSuiAddress();

    const encryptedPrivateKey = await encrypt(
      Buffer.from(keypair.export().privateKey).toString('hex'),
      password
    );

    const wallet = {
      address,
      publicKey: keypair.getPublicKey().toBase64(),
      privateKey: encryptedPrivateKey,
      mnemonic: await encrypt(mnemonic, password),
      name: `Wallet ${Date.now()}`
    };

    await this.saveWallet(wallet);

    return {
      address,
      mnemonic
    };
  }

  async importFromMnemonic(mnemonic: string, password: string): Promise<string> {
    const seed = await mnemonicToSeed(mnemonic);
    const keypair = Ed25519Keypair.fromSeed(seed);
    const address = keypair.getPublicKey().toSuiAddress();

    const wallet = {
      address,
      publicKey: keypair.getPublicKey().toBase64(),
      privateKey: await encrypt(
        Buffer.from(keypair.export().privateKey).toString('hex'),
        password
      ),
      mnemonic: await encrypt(mnemonic, password),
      name: `Imported Wallet ${Date.now()}`
    };

    await this.saveWallet(wallet);
    return address;
  }

  private async saveWallet(wallet: Wallet): Promise<void> {
    const wallets = await this.getWallets();
    wallets.push(wallet);
    await chrome.storage.local.set({ [this.WALLET_STORAGE_KEY]: wallets });
    await this.setCurrentWallet(wallet);
  }

  private async getWallets(): Promise<Wallet[]> {
    const result = await chrome.storage.local.get(this.WALLET_STORAGE_KEY);
    return result[this.WALLET_STORAGE_KEY] || [];
  }

  private async setCurrentWallet(wallet: Wallet): Promise<void> {
    await chrome.storage.local.set({ [this.CURRENT_WALLET_KEY]: wallet });
  }
}