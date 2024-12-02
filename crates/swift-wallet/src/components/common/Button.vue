<template>
    <button
      :class="[
        'swift-button',
        `swift-button--${type}`,
        { 'swift-button--loading': loading }
      ]"
      :disabled="disabled || loading"
      @click="$emit('click')"
    >
      <span v-if="loading" class="loading-spinner"></span>
      <slot></slot>
    </button>
  </template>
  
  <script lang="ts">
  import { defineComponent } from 'vue'
  
  export default defineComponent({
    name: 'SwiftButton',
    props: {
      type: {
        type: String,
        default: 'primary',
        validator: (value: string) => {
          return ['primary', 'secondary', 'danger'].includes(value)
        }
      },
      loading: {
        type: Boolean,
        default: false
      },
      disabled: {
        type: Boolean,
        default: false
      }
    }
  })
  </script>
  
  <style scoped>
  .swift-button {
    padding: 12px 24px;
    border-radius: 8px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.3s ease;
    border: none;
    outline: none;
    position: relative;
  }
  
  .swift-button--primary {
    background: #3498db;
    color: white;
  }
  
  .swift-button--secondary {
    background: #ecf0f1;
    color: #2c3e50;
  }
  
  .swift-button--danger {
    background: #e74c3c;
    color: white;
  }
  
  .swift-button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
  
  .loading-spinner {
    display: inline-block;
    width: 16px;
    height: 16px;
    border: 2px solid #ffffff;
    border-radius: 50%;
    border-top-color: transparent;
    animation: spin 1s linear infinite;
    margin-right: 8px;
  }
  
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
  </style>