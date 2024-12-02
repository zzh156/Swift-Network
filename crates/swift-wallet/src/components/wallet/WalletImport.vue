<template>
    <div class="wallet-import">
      <h2>导入钱包</h2>
      
      <div class="import-type-selector">
        <SwiftButton
          :type="importType === 'mnemonic' ? 'primary' : 'secondary'"
          @click="importType = 'mnemonic'"
        >
          助记词导入
        </SwiftButton>
        <SwiftButton
          :type="importType === 'privateKey' ? 'primary' : 'secondary'"
          @click="importType = 'privateKey'"
        >
          私钥导入
        </SwiftButton>
      </div>
  
      <form @submit.prevent="handleImport">
        <template v-if="importType === 'mnemonic'">
          <SwiftInput
            v-model="mnemonic"
            type="text"
            label="助记词"
            placeholder="请输入12/24位助记词，用空格分隔"
            id="mnemonic"
            :error="mnemonicError"
          />
        </template>
  
        <template v-else>
          <SwiftInput
            v-model="privateKey"
            type="text"
            label="私钥"
            placeholder="请输入私钥"
            id="private-key"
            :error="privateKeyError"
          />
        </template>
  
        <SwiftInput
          v-model="password"
          type="password"
          label="设置密码"
          placeholder="请设置钱包密码"
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
          导入钱包
        </SwiftButton>
      </form>
    </div>
  </template>
  
  <script lang="ts">
  import { defineComponent, ref, computed } from 'vue'
  import { useRouter } from 'vue-router'
  import { useWalletStore } from '../../store/wallet'
  import SwiftButton from '../common/Button.vue'
  import SwiftInput from '../common/Input.vue'
  
  export default defineComponent({
    name: 'WalletImport',
    components: {
      SwiftButton,
      SwiftInput
    },
    setup() {
      const router = useRouter()
      const walletStore = useWalletStore()
      
      const importType = ref<'mnemonic' | 'privateKey'>('mnemonic')
      const mnemonic = ref('')
      const privateKey = ref('')
      const password = ref('')
      const confirmPassword = ref('')
      const loading = ref(false)
  
      const mnemonicError = ref('')
      const privateKeyError = ref('')
      const passwordError = ref('')
      const confirmPasswordError = ref('')
  
      const isFormValid = computed(() => {
        if (importType.value === 'mnemonic') {
          return mnemonic.value.trim().split(' ').length >= 12 &&
                 password.value.length >= 8 &&
                 password.value === confirmPassword.value
        } else {
          return privateKey.value.length > 0 &&
                 password.value.length >= 8 &&
                 password.value === confirmPassword.value
        }
      })
  
      const validateForm = () => {
        mnemonicError.value = ''
        privateKeyError.value = ''
        passwordError.value = ''
        confirmPasswordError.value = ''
  
        if (importType.value === 'mnemonic') {
          const words = mnemonic.value.trim().split(' ')
          if (words.length !== 12 && words.length !== 24) {
            mnemonicError.value = '请输入12或24位助记词'
            return false
          }
        } else {
          if (!privateKey.value) {
            privateKeyError.value = '请输入私钥'
            return false
          }
        }
  
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
  
      const handleImport = async () => {
        if (!validateForm()) return
  
        loading.value = true
        try {
          if (importType.value === 'mnemonic') {
            const result = await walletStore.importWallet(mnemonic.value, password.value)
            if (result.success) {
              router.push('/wallet-details')
            } else {
              mnemonicError.value = result.error || '导入失败'
            }
          } else {
            const result = await walletStore.importWalletFromPrivateKey(privateKey.value, password.value)
            if (result.success) {
              router.push('/wallet-details')
            } else {
              privateKeyError.value = result.error || '导入失败'
            }
          }
        } catch (error) {
          console.error('导入钱包失败:', error)
        } finally {
          loading.value = false
        }
      }
  
      return {
        importType,
        mnemonic,
        privateKey,
        password,
        confirmPassword,
        loading,
        mnemonicError,
        privateKeyError,
        passwordError,
        confirmPasswordError,
        isFormValid,
        handleImport
      }
    }
  })
  </script>
  
  <style scoped>
  .wallet-import {
    max-width: 480px;
    margin: 0 auto;
    padding: 24px;
  }
  
  .import-type-selector {
    display: flex;
    gap: 12px;
    margin-bottom: 24px;
  }
  
  .import-type-selector button {
    flex: 1;
  }
  </style>