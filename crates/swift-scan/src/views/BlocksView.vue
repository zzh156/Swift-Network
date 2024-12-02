<script setup lang="ts">
import { ref, onMounted } from 'vue'

interface Block {
  height: number
  hash: string
  timestamp: number
  transactions: number
  proposer: string
  size: number
}

const blocks = ref<Block[]>([])
const loading = ref(true)

const fetchBlocks = async () => {
  loading.value = true
  try {
    // TODO: 实现从API获取区块列表
    // 示例数据
    blocks.value = [
      {
        height: 1000000,
        hash: '0x1234...5678',
        timestamp: Date.now(),
        transactions: 100,
        proposer: '0xabcd...efgh',
        size: 1024
      }
    ]
  } catch (error) {
    console.error('获取区块列表失败:', error)
  } finally {
    loading.value = false
  }
}

onMounted(fetchBlocks)
</script>

<template>
  <div class="blocks">
    <h1>区块列表</h1>
    
    <div class="filters">
      <input type="text" placeholder="搜索区块高度或哈希..." class="search-input">
    </div>

    <div class="blocks-table" v-if="!loading">
      <table>
        <thead>
          <tr>
            <th>区块高度</th>
            <th>区块哈希</th>
            <th>时间</th>
            <th>交易数</th>
            <th>提议者</th>
            <th>区块大小</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="block in blocks" :key="block.hash">
            <td>{{ block.height }}</td>
            <td>{{ block.hash }}</td>
            <td>{{ new Date(block.timestamp).toLocaleString() }}</td>
            <td>{{ block.transactions }}</td>
            <td>{{ block.proposer }}</td>
            <td>{{ block.size }} bytes</td>
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
.blocks {
  padding: 1rem;
}

.filters {
  margin-bottom: 1rem;
}

.search-input {
  padding: 0.5rem;
  border: 1px solid #ddd;
  border-radius: 4px;
  width: 300px;
}

.blocks-table {
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

.loading {
  text-align: center;
  padding: 2rem;
}
</style>