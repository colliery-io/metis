<template>
  <div v-if="isVisible" class="sync-indicator" :title="tooltipText">
    <!-- In Progress -->
    <button
      v-if="syncStatus?.in_progress"
      class="sync-button syncing"
      disabled
    >
      <div class="sync-spinner" />
      <span>Syncing...</span>
    </button>

    <!-- Error State -->
    <button
      v-else-if="syncStatus?.last_error"
      class="sync-button error"
      @click="handleSync"
    >
      <div class="status-dot error-dot" />
      <span>Sync Error</span>
    </button>

    <!-- Synced State -->
    <button
      v-else-if="syncStatus?.last_synced"
      class="sync-button synced"
      @click="handleSync"
      :disabled="isSyncing"
    >
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
        <polyline points="20 6 9 17 4 12" />
      </svg>
      <span>{{ relativeTime }}</span>
    </button>

    <!-- Never Synced (upstream configured but no sync yet) -->
    <button
      v-else
      class="sync-button ready"
      @click="handleSync"
      :disabled="isSyncing"
    >
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M23 4v6h-6M1 20v-6h6" />
        <path d="M3.51 9a9 9 0 0114.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0020.49 15" />
      </svg>
      <span>Sync</span>
    </button>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import type { SyncStatus } from '../lib/tauri-api'
import { MetisAPI } from '../lib/tauri-api'
import { emit } from '@tauri-apps/api/event'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

const syncStatus = ref<SyncStatus | null>(null)
const isVisible = ref(false)
const isSyncing = ref(false)
let pollInterval: ReturnType<typeof setInterval> | null = null
let unlistenSyncCompleted: UnlistenFn | null = null

const relativeTime = computed(() => {
  if (!syncStatus.value?.last_synced) return 'Never'
  const now = new Date()
  const synced = new Date(syncStatus.value.last_synced)
  const diffMs = now.getTime() - synced.getTime()
  const diffSecs = Math.floor(diffMs / 1000)
  const diffMins = Math.floor(diffSecs / 60)
  const diffHours = Math.floor(diffMins / 60)

  if (diffSecs < 60) return 'Just now'
  if (diffMins < 60) return `${diffMins}m ago`
  if (diffHours < 24) return `${diffHours}h ago`
  return 'Yesterday'
})

const tooltipText = computed(() => {
  if (syncStatus.value?.in_progress) return 'Sync in progress...'
  if (syncStatus.value?.last_error) return `Error: ${syncStatus.value.last_error}`
  if (syncStatus.value?.last_result_summary) return syncStatus.value.last_result_summary
  if (syncStatus.value?.last_synced) return `Last synced: ${relativeTime.value}`
  return 'Click to sync with central repository'
})

const checkUpstream = async () => {
  try {
    const configured = await MetisAPI.isUpstreamConfigured()
    isVisible.value = configured
    if (configured) {
      await refreshStatus()
    }
  } catch {
    isVisible.value = false
  }
}

const refreshStatus = async () => {
  try {
    syncStatus.value = await MetisAPI.getSyncStatus()
  } catch {
    // Silently fail status checks
  }
}

const handleSync = async () => {
  if (isSyncing.value || syncStatus.value?.in_progress) return

  isSyncing.value = true
  try {
    const result = await MetisAPI.syncWorkspace()
    emit('show-toast', {
      message: result.summary,
      type: 'success'
    })
  } catch (error) {
    emit('show-toast', {
      message: error instanceof Error ? error.message : String(error),
      type: 'error'
    })
  } finally {
    isSyncing.value = false
    await refreshStatus()
  }
}

onMounted(async () => {
  await checkUpstream()

  // Poll status every 30 seconds
  pollInterval = setInterval(refreshStatus, 30000)

  // Listen for sync-completed events to refresh status
  unlistenSyncCompleted = await listen('sync-completed', async () => {
    await refreshStatus()
  })
})

onUnmounted(() => {
  if (pollInterval) clearInterval(pollInterval)
  if (unlistenSyncCompleted) unlistenSyncCompleted()
})

// Expose for parent component to trigger re-check
defineExpose({ checkUpstream, handleSync })
</script>

<style scoped>
.sync-indicator {
  display: flex;
  align-items: center;
}

.sync-button {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  background: transparent;
  border: 1px solid var(--color-border-primary);
  border-radius: 6px;
  font-family: var(--font-mono);
  font-size: 11px;
  font-weight: 500;
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all 0.2s ease;
  white-space: nowrap;
}

.sync-button:hover:not(:disabled) {
  background: var(--color-background-tertiary);
  border-color: var(--color-border-secondary);
  color: var(--color-text-primary);
}

.sync-button:disabled {
  cursor: default;
  opacity: 0.7;
}

.sync-button.syncing {
  color: var(--color-interactive-primary);
  border-color: var(--color-interactive-primary);
}

.sync-button.error {
  color: var(--color-border-error);
  border-color: var(--color-border-error);
}

.sync-button.error:hover {
  background: rgba(var(--color-border-error-rgb, 229, 115, 115), 0.1);
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.error-dot {
  background: var(--color-border-error);
}

.sync-spinner {
  width: 14px;
  height: 14px;
  border: 2px solid var(--color-border-primary);
  border-top-color: var(--color-interactive-primary);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  flex-shrink: 0;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>
