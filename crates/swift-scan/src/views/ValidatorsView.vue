<script setup lang="ts">
import { ref, onMounted } from 'vue'

interface Validator {
  address: string
  name: string
  votingPower: number
  commission: number
  uptime: number
  delegators: number
}

const validators = ref<Validator[]>([])
const loading = ref(true)

const fetchValidators = async () => {
  loading.value = true
  try {
    // TODO: 实现从API获取验证者列表
    validators.value = [
      {
        address: '0x1234...5678',
        name: 'Validator 1',
        votingPower: 1000000,
        commission: 0.05,
        uptime: 0.999,
        delegators: 100
      }
    ]
  } catch (error) {
    console.error('获取验证者列表失败:', error)
  } finally {
    loading.value = false
  }
}

onMounted(fetchValidators)
</script>

<template>
  <div class="validators">
    <h1>验证者列表</h1>

    <div class="validators-grid" v-if="!loading">
      <div v-for="validator in validators" 
           :key="validator.address" 
           class="validator-card">
        <h3>{{ validator.name }}</h3>
        <div class="validator-info">
          <p>地址: {{ validator.address }}</p>
          <p>投票权重: {{ validator.votingPower }}</p>
          <p>佣金率: {{ validator.commission * 100 }}%</p>
          <p>在线率: {{ validator.uptime * 100 }}%</p>
          <p>委托人数: {{ validator.delegators }}</p>
        </div>
      </div>
    </div>

    <div v-else class="loading">
      加载中...
    </div>
  </div>
</template>

<style scoped>
.validators {
  padding: 1rem;
}

.validators-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 1rem;
}

.validator-card {
  background: white;
  border-radius: 8px;
  padding: 1rem;
  box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}

.validator-info {
  margin-top: 1rem;
}

.validator-info p {
  margin: 0.5rem 0;
  color: #666;
}

.loading {
  text-align: center;
  padding: 2rem;
}
</style>