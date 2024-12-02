<template>
    <div class="transaction-form">
      <h3>发送交易</h3>
      
      <form @submit.prevent="handleSubmit">
        <SwiftInput
          v-model="recipient"
          type="text"
          label="接收地址"
          placeholder="请输入接收地址"
          id="recipient"
          :error="recipientError"
        />
  
        <div class="amount-input-container">
          <SwiftInput
            v-model="amount"
            type="number"
            label="发送数量"
            placeholder="请输入发送数量"
            id="amount"
            :error="amountError"
          />
          <div class="balance-info">
            可用余额: {{ availableBalance }} SWIFT
          </div>
        </div>
  
        <div class="gas-settings" v-if="showAdvanced">
          <SwiftInput
            v-model="gasPrice"
            type="number"
            label="Gas Price (SWIFT)"
            placeholder="自定义 Gas Price"
            id="gas-price"
          />
          
          <SwiftInput
            v-model="gasLimit"
            type="number"
            label="Gas Limit"
            placeholder="自定义 Gas Limit"
            id="gas-limit"
          />
        </div>
  
        <div class="advanced-toggle">
          <button 
            type="button" 
            class="toggle-button"
            @click="showAdvanced = !showAdvanced"
          >
            {{ showAdvanced ? '隐藏高级选项' : '显示高级选项' }}
          </button>
        </div>
  
        <div class="transaction-summary" v-if="isFormValid">
          <div class="summary-item">
            <span>交易金额</span>
            <span>{{ amount }} SWIFT</span>
          </div>
          <div class="summary-item">
            <span>预估 Gas 费用</span>
            <span>{{ estimatedGasFee }} SWIFT</span>
          </div>
          <div class="summary-item total">
            <span>总计</span>
            <span>{{ totalAmount }} SWIFT</span>
          </div>
        </div>
  
        <div class="action-buttons">
          <SwiftButton
            type="secondary"
            @click="$emit('cancel')"
          >
            取消
          </SwiftButton>
          
          <SwiftButton
            type="primary"
            :loading="loading"
            :disabled="!isFormValid || loading"
          >
            确认发送
          </SwiftButton>
        </div>
      </form>
    </div>
  </template>
  
  <script lang="ts">
  import { defineComponent, ref, computed } from 'vue'
  import { useWalletStore } from '../../store/wallet'
  import SwiftButton from '../common/Button.vue'
  import SwiftInput from '../common/Input.vue'
  import { validateAddress } from '../../utils/validation'
  
  export default defineComponent({
    name: 'TransactionForm',
    components: {
      SwiftButton,
      SwiftInput
    },
    emits: ['cancel', 'success'],
    setup(props, { emit }) {
      const walletStore = useWalletStore()
      
      const recipient = ref('')
      const amount = ref('')
      const gasPrice = ref('')
      const gasLimit = ref('')
      const loading = ref(false)
      const showAdvanced = ref(false)
      
      const recipientError = ref('')
      const amountError = ref('')
  
      const availableBalance = computed(() => {
        return parseFloat(walletStore.balance).toLocaleString()
      })
  
      const estimatedGasFee = computed(() => {
        if (gasPrice.value && gasLimit.value) {
          return (parseFloat(gasPrice.value) * parseFloat(gasLimit.value)).toFixed(6)
        }
        return '0.000021' // 默认估算值
      })
  
      const totalAmount = computed(() => {
        const total = parseFloat(amount.value || '0') + parseFloat(estimatedGasFee.value)
        return total.toFixed(6)
      })
  
      const isFormValid = computed(() => {
        return validateAddress(recipient.value) &&
               parseFloat(amount.value) > 0 &&
               parseFloat(totalAmount.value) <= parseFloat(walletStore.balance)
      })
  
      const validateForm = () => {
        recipientError.value = ''
        amountError.value = ''
  
        if (!validateAddress(recipient.value)) {
          recipientError.value = '请输入有效的接收地址'
          return false
        }
  
        if (parseFloat(amount.value) <= 0) {
          amountError.value = '请输入有效的发送数量'
          return false
        }
  
        if (parseFloat(totalAmount.value) > parseFloat(walletStore.balance)) {
          amountError.value = '余额不足'
          return false
        }
  
        return true
      }
  
      const handleSubmit = async () => {
        if (!validateForm()) return
  
        loading.value = true
        try {
          const tx = {
            to: recipient.value,
            amount: amount.value,
            gasPrice: gasPrice.value || undefined,
            gasLimit: gasLimit.value || undefined
          }
  
          const result = await walletStore.sendTransaction(tx)
          if (result.success) {
            emit('success', result.txHash)
          } else {
            throw new Error(result.error)
          }
        } catch (error) {
          console.error('交易发送失败:', error)
          // 这里可以添加错误提示
        } finally {
          loading.value = false
        }
      }
  
      return {
        recipient,
        amount,
        gasPrice,
        gasLimit,
        loading,
        showAdvanced,
        recipientError,
        amountError,
        availableBalance,
        estimatedGasFee,
        totalAmount,
        isFormValid,
        handleSubmit
      }
    }
  })
  </script>
  
  <style scoped>
  .transaction-form {
    padding: 20px;
    max-width: 480px;
    margin: 0 auto;
  }
  
  .amount-input-container {
    position: relative;
    margin-bottom: 20px;
  }
  
  .balance-info {
    position: absolute;
    right: 0;
    top: 0;
    font-size: 12px;
    color: #666;
  }
  
  .gas-settings {
    margin-top: 16px;
    padding: 16px;
    background: #f8f9fa;
    border-radius: 8px;
  }
  
  .advanced-toggle {
    text-align: center;
    margin: 16px 0;
  }
  
  .toggle-button {
    background: none;
    border: none;
    color: #3498db;
    cursor: pointer;
    font-size: 14px;
  }
  
  .transaction-summary {
    margin: 24px 0;
    padding: 16px;
    background: #f8f9fa;
    border-radius: 8px;
  }
  
  .summary-item {
    display: flex;
    justify-content: space-between;
    margin-bottom: 8px;
    font-size: 14px;
  }
  
  .summary-item.total {
    margin-top: 16px;
    padding-top: 16px;
    border-top: 1px solid #dcdfe6;
    font-weight: bold;
  }
  
  .action-buttons {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
    margin-top: 24px;
  }
  </style>