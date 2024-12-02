<script setup lang="ts">
import { onMounted } from 'vue'
import { useBlockchainStore } from '@/stores/blockchain'

const store = useBlockchainStore()

onMounted(async () => {
  await Promise.all([
    store.fetchLatestBlocks(),
    store.fetchLatestTransactions(),
    store.fetchNetworkStats()
  ])
})
</script>

<template>
  <div class="home">
    <section class="network-stats">
      <div class="stat-card">
        <h3>总交易数</h3>
        <p>{{ store.networkStats.totalTransactions }}</p>
      </div>
      <div class="stat-card">
        <h3>TPS</h3>
        <p>{{ store.networkStats.tps }}</p>
      </div>
      <div class="stat-card">
        <h3>活跃验证者</h3>
        <p>{{ store.networkStats.activeValidators }}</p>
      </div>
      <div class="stat-card">
        <h3>当前纪元</h3>
        <p>{{ store.networkStats.epochNumber }}</p>
      </div>
    </section>

    <div class="content-grid">
      <section class="latest-blocks">
        <h2>最新区块</h2>
        <!-- 区块列表 -->
      </section>

      <section class="latest-transactions">
        <h2>最新交易</h2>
        <!-- 交易列表 -->
      </section>
    </div>
  </div>
</template>

<style scoped>
.home {
  padding: 1rem;
}

.network-stats {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 1rem;
  margin-bottom: 2rem;
}

.stat-card {
  background: white;
  padding: 1.5rem;
  border-radius: 8px;
  box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}

.content-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 2rem;
}

@media (max-width: 768px) {
  .content-grid {
    grid-template-columns: 1fr;
  }
}
</style>
