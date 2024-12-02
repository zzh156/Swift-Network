<template>
    <div class="wallet-details">
      <header class="wallet-header">
        <h2>我的钱包</h2>
        <div class="network-selector">
          <select v-model="selectedNetwork">
            <option 
              v-for="network in networks" 
              :key="network.id" 
              :value="network.id"
            >
              {{ network.name }}
            </option>
          </select>
        </div>
      </header>
  
      <WalletBalance
        @send="showSendForm = true"
        @receive="showReceiveModal = true"
      />
  
      <div class="transactions-list">
        <h3>最近交易</h3>
        <div v-if="transactions.length > 0">
          <div 
            v-for="tx in transactions" 
            :key="tx.hash"
            class="transaction-item"
          >
            <div class="tx-type" :class="tx.type">
              {{ tx.type === 'send' ? '发送' : '接收' }}
            </div>
            <div class="tx-details">
              <div class="tx-amount">{{ tx.amount }} SWIFT</div>
              <div class="tx-date">{{ formatDate(tx.timestamp) }}</div>
            </div>
            <div class="tx-status" :class="tx.status">
              {{ tx.status }}
            </div>
          </div>
        </div>
        <div v-else class="no-transactions">
          暂无交易记录
        </div>
      </div>
  
      <Modal v-if="showSendForm" @close="showSendForm = false">
        <TransactionForm
          @cancel="showSendForm = false"
          @success="handleTransactionSuccess"
        />
      </Modal>
  
      <Modal v-if="showReceiveModal" @close="showReceiveModal = false">
        <div class="receive-modal">
          <h3>接收 SWIFT</h3>
          <div class="qr-code">
            <!-- 这里添加二维码组件 -->
          </div>
          <div class="address-display">
            <p>钱包地址</p>
            <div class="address-box">
              {{ currentWallet?.address }}
              <button @click="copyAddress" class="copy-button">
                复制
              </button>
            </div>
          </div>
        </div>
      </Modal>
    </div>
  </template>
  
  <script lang="ts">
  import { defineComponent, ref, computed, onMounted } from 'vue'
  import { useWalletStore } from '../store/wallet'
  import WalletBalance from '../components/wallet/WalletBalance.vue'
  import TransactionForm from '../components/wallet/TransactionForm.vue'
  import Modal from '../components/common/Modal.vue'
  
  export default defineComponent({
    name: 'WalletDetailsView',
    components: {
      WalletBalance,
      TransactionForm,
      Modal
    },
    setup() {
      const walletStore = useWalletStore()
      const showSendForm = ref(false)
      const showReceiveModal = ref(false)
      const selectedNetwork = ref('mainnet')
      
      const networks = [
        { id: 'mainnet', name: 'Swift 主网' },
        { id: 'testnet', name: 'Swift 测试网' }
      ]
  
      const transactions = ref([])
      const currentWallet = computed(() => walletStore.currentWallet)
  
      const formatDate = (timestamp: number) => {
        return new Date(timestamp).toLocaleString()
      }
  
      const copyAddress = async () => {
        if (currentWallet.value?.address) {
          await navigator.clipboard.writeText(currentWallet.value.address)
          // 可以添加提示
        }
      }
  
      const handleTransactionSuccess = (txHash: string) => {
        showSendForm.value = false
        // 可以添加成功提示
        // 刷新交易列表
      }
  
      onMounted(async () => {
        // 加载交易历史
      })
  
      return {
        showSendForm,
        showReceiveModal,
        selectedNetwork,
        networks,
        transactions,
        currentWallet,
        formatDate,
        copyAddress,
        handleTransactionSuccess
      }
    }
  })
  </script>
  
  <style scoped>
  .wallet-details {
    max-width: 768px;
    margin: 0 auto;
    padding: 20px;
  }
  
  .wallet-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 24px;
  }
  
  .network-selector select {
    padding: 8px;
    border-radius: 8px;
    border: 1px solid #dcdfe6;
  }
  
  .transactions-list {
    margin-top: 32px;
  }
  
  .transaction-item {
    display: flex;
    align-items: center;
    padding: 16px;
    border-bottom: 1px solid #eee;
  }
  
  .tx-type {
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 12px;
    margin-right: 12px;
  }
  
  .tx-type.send {
    background: #ffeaea;
    color: #e74c3c;
  }
  
  .tx-type.receive {
    background: #eafaf1;
    color: #27ae60;
  }
  
  .tx-details {
    flex: 1;
  }
  
  .tx-amount {
    font-weight: 500;
  }
  
  .tx-date {
    font-size: 12px;
    color: #666;
  }
  
  .tx-status {
    font-size: 12px;
  }
  
  .tx-status.pending {
    color: #f39c12;
  }
  
  .tx-status.confirmed {
    color: #27ae60;
  }
  
  .tx-status.failed {
    color: #e74c3c;
  }
  
  .no-transactions {
    text-align: center;
    padding: 32px;
    color: #666;
  }
  
  .receive-modal {
    padding: 24px;
    text-align: center;
  }
  
  .qr-code {
    margin: 24px 0;
    padding: 16px;
    background: #fff;
    display: inline-block;
  }
  
  .address-display {
    margin-top: 16px;
  }
  
  .address-box {
    background: #f8f9fa;
    padding: 12px;
    border-radius: 8px;
    margin-top: 8px;
    word-break: break-all;
    position: relative;
  }
  
  .copy-button {
    position: absolute;
    right: 8px;
    top: 50%;
    transform: translateY(-50%);
    background: none;
    border: none;
    color: #3498db;
    cursor: pointer;
  }
  </style>