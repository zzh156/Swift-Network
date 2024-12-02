<template>
    <div class="wallet-create">
      <h2>创建新钱包</h2>
      <form @submit.prevent="handleCreate">
        <SwiftInput
          v-model="password"
          type="password"
          label="设置密码"
          placeholder="请输入钱包密码"
          id="password"
          :error="passwordError"
        />
        <SwiftInput
          v-model="confirmPassword"
          type="password"
          label="确认密码"
          placeholder="请再次输入密码"
          id="confirm-password"
          :error="confirmPasswordError"
        />
        <SwiftButton
          type="primary"
          :loading="loading"
          :disabled="!isFormValid"
        >
          创建钱包
        </SwiftButton>
      </form>
  
      <div v-if="mnemonic" class="mnemonic-container">
        <h3>请保存您的助记词</h3>
        <div class="mnemonic-words">
          <div
            v-for="(word, index) in mnemonicArray"
            :key="index"
            class="mnemonic-word"
          >
            <span class="word-index">{{ index + 1 }}.</span>
            <span class="word">{{ word }}</span>
          </div>
        </div>
        <div class="warning">
          警告：请将助记词保存在安全的地方，切勿分享给他人！
        </div>
        <SwiftButton
          type="primary"
          @click="handleConfirm"
        >
          我已安全保存助记词
        </SwiftButton>
      </div>
    </div>
  </template>
  
  <script lang="ts">
  import { defineComponent, ref, computed } from 'vue'
  import { useRouter } from 'vue-router'
  import { useWalletStore } from '../../store/wallet'
  import SwiftButton from '../common/Button.vue'
  import SwiftInput from '../common/Input.vue'
  
  export default defineComponent({
    name: 'WalletCreate',
    components: {
      SwiftButton,
      SwiftInput
    },
    setup() {
      const router = useRouter()
      const walletStore = useWalletStore()
      
      const password = ref('')
      const confirmPassword = ref('')
      const loading = ref(false)
      const mnemonic = ref('')
      const passwordError = ref('')
      const confirmPasswordError = ref('')
  
      const mnemonicArray = computed(() => {
        return mnemonic.value ? mnemonic.value.split(' ') : []
      })
  
      const isFormValid = computed(() => {
        return password.value.length >= 8 && 
               password.value === confirmPassword.value
      })
  
      const validateForm = () => {
        passwordError.value = ''
        confirmPasswordError.value = ''
  
        if (password.value.length < 8) {
          passwordError.value = '密码长度至少8位'
          return false
        }
  
        if (password.value !== confirmPassword.value) {
          confirmPasswordError.value = '两次输入的密码不一致'
          return false
        }
  
        return true
      }
  
      const handleCreate = async () => {
        if (!validateForm()) return
  
        loading.value = true
        try {
          const result = await walletStore.createWallet(password.value)
          if (result.success) {
            mnemonic.value = result.mnemonic
          } else {
            throw new Error(result.error)
          }
        } catch (error) {
          console.error('创建钱包失败:', error)
        } finally {
          loading.value = false
        }
      }
  
      const handleConfirm = () => {
        router.push('/wallet-details')
      }
  
      return {
        password,
        confirmPassword,
        loading,
        mnemonic,
        mnemonicArray,
        passwordError,
        confirmPasswordError,
        isFormValid,
        handleCreate,
        handleConfirm
      }
    }
  })
  </script>
  
  <style scoped>
  .wallet-create {
    max-width: 480px;
    margin: 0 auto;
    padding: 24px;
  }
  
  .mnemonic-container {
    margin-top: 24px;
    padding: 16px;
    background: #f8f9fa;
    border-radius: 8px;
  }
  
  .mnemonic-words {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 12px;
    margin: 16px 0;
  }
  
  .mnemonic-word {
    display: flex;
    align-items: center;
    padding: 8px;
    background: white;
    border-radius: 4px;
    border: 1px solid #dcdfe6;
  }
  
  .word-index {
    color: #666;
    margin-right: 8px;
    font-size: 12px;
  }
  
  .warning {
    color: #e74c3c;
    margin: 16px 0;
    font-size: 14px;
    text-align: center;
  }
  </style>