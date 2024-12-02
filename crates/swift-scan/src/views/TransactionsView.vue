<script setup lang="ts">
import { ref, onMounted } from 'vue'

interface Transaction {
  hash: string
  timestamp: number
  sender: string
  receiver: string
  amount: string
  status: 'success' | 'failed'
  gasUsed: number
}

const transactions = ref<Transaction[]>([])
const loading = ref(true)

const fetchTransactions = async () => {
  loading.value = true
  try {
    // TODO: 实现从API获取交易列表
    // 示例数据
    transactions.value = [
      {
        hash: '0x1234...5678',
        timestamp: Date.now(),
        sender: '0xabcd...efgh',
        receiver: '0xijkl...mnop',
        amount: '100 SWIFT',
        status: 'success',
        gasUsed: 1000
      }
    ]
  } catch (error) {
    console.error('获取交易列表失败:', error)
  } finally {
    loading.value = false
  }
}

onMounted(fetchTransactions)
</script>

<template>
  <div class="transactions">
    <h1>交易列表</h1>
    
    <div class="filters">
      <input type="text" placeholder="搜索交易哈希..." class="search-input">
      <select class="filter-select">
        <option value="all">所有状态</option>
        <option value="success">成功</option>
        <option value="failed">失败</option>
      </select>
    </div>

    <div class="transactions-table" v-if="!loading">
      <table>
        <thead>
          <tr>
            <th>交易哈希</th>
            <th>时间</th>
            <th>发送方</th>
            <th>接收方</th>
            <th>数量</th>
            <th>状态</th>
            <th>Gas 消耗</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="tx in transactions" :key="tx.hash">
            <td>{{ tx.hash }}</td>
            <td>{{ new Date(tx.timestamp).toLocaleString() }}</td>
            <td>{{ tx.sender }}</td>
            <td>{{ tx.receiver }}</td>
            <td>{{ tx.amount }}</td>
            <td>
              <span :class="['status', tx.status]">
                {{ tx.status === 'success' ? '成功' : '失败' }}
              </span>
            </td>
            <td>{{ tx.gasUsed }}</td>
          </tr>
        </tbody>
      </table>
    </div>

    <div v-else class="loading">
      加载中...
    </div>
  </div>
</template>

<style scoped>
.transactions {
  padding: 1rem;
}

.filters {
  display: flex;
  gap: 1rem;
  margin-bottom: 1rem;
}

.search-input,
.filter-select {
  padding: 0.5rem;
  border: 1px solid #ddd;
  border-radius: 4px;
}

.transactions-table {
  background: white;
  border-radius: 8px;
  box-shadow: 0 2px 4px rgba(0,0,0,0.1);
  overflow-x: auto;
}

table {
  width: 100%;
  border-collapse: collapse;
}

th, td {
  padding: 1rem;
  text-align: left;
  border-bottom: 1px solid #eee;
}

th {
  background: #f8f9fa;
  font-weight: 600;
}

.status {
  padding: 0.25rem 0.5rem;
  border-radius: 4px;
  font-size: 0.875rem;
}

.status.success {
  background: #e6f4ea;
  color: #1e7e34;
}

.status.failed {
  background: #fde7e9;
  color: #dc3545;
}

.loading {
  text-align: center;
  padding: 2rem;
}
</style>