<script setup lang="ts">
import { RouterLink, RouterView } from 'vue-router'
</script>

<template>
  <header>
    <div class="header-content">
      <div class="logo-container">
        <img alt="Swift logo" class="logo" src="@/assets/logo.svg" width="40" height="40" />
        <span class="logo-text">Swift Scan</span>
      </div>

      <nav>
        <RouterLink to="/">首页</RouterLink>
        <RouterLink to="/transactions">交易</RouterLink>
        <RouterLink to="/blocks">区块</RouterLink>
        <RouterLink to="/addresses">地址</RouterLink>
        <RouterLink to="/validators">验证者</RouterLink>
      </nav>

      <div class="search-container">
        <input 
          type="text" 
          placeholder="搜索地址 / 交易哈希 / 区块高度..."
          class="search-input"
        >
      </div>
    </div>
  </header>

  <main>
    <RouterView />
  </main>
</template>

<style scoped>
header {
  background: #ffffff;
  box-shadow: 0 2px 4px rgba(0,0,0,0.1);
  position: fixed;
  top: 0;
  width: 100%;
  z-index: 100;
}

.header-content {
  max-width: 1200px;
  margin: 0 auto;
  padding: 1rem;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.logo-container {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.logo-text {
  font-size: 1.5rem;
  font-weight: bold;
  color: #2c3e50;
}

nav {
  display: flex;
  gap: 2rem;
}

nav a {
  color: #666;
  text-decoration: none;
  font-size: 1rem;
  padding: 0.5rem 0;
}

nav a.router-link-active {
  color: #3498db;
  border-bottom: 2px solid #3498db;
}

.search-container {
  flex: 1;
  max-width: 400px;
  margin-left: 2rem;
}

.search-input {
  width: 100%;
  padding: 0.5rem 1rem;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 0.9rem;
}

main {
  margin-top: 80px;
  padding: 2rem;
  max-width: 1200px;
  margin-left: auto;
  margin-right: auto;
}
</style>
