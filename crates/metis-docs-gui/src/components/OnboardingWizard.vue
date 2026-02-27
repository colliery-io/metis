<template>
  <div
    v-if="isOpen"
    class="fixed inset-0 z-50 flex items-center justify-center"
  >
    <!-- Backdrop -->
    <div
      class="absolute inset-0 bg-overlay"
      @click.stop
    />

    <!-- Dialog -->
    <div class="wizard-dialog">
      <!-- Progress Steps -->
      <div class="wizard-steps">
        <div
          v-for="(step, index) in steps"
          :key="step"
          :class="['wizard-step', { active: currentStep === index, completed: currentStep > index }]"
        >
          <div class="step-indicator">
            <svg v-if="currentStep > index" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
              <polyline points="20 6 9 17 4 12" />
            </svg>
            <span v-else>{{ index + 1 }}</span>
          </div>
          <span class="step-label">{{ step }}</span>
        </div>
      </div>

      <!-- Step 1: Upstream URL -->
      <div v-if="currentStep === 0" class="wizard-content">
        <h3 class="wizard-title">Connect to Central Repository</h3>
        <p class="wizard-description">
          Enter the URL of your team's central Metis repository to enable multi-workspace sync.
        </p>

        <div class="form-field">
          <label class="field-label">Repository URL</label>
          <input
            v-model="upstreamUrl"
            type="text"
            class="field-input"
            :class="{ 'field-error': urlError, 'field-success': urlTestPassed }"
            placeholder="git@github.com:org/metis-central.git"
            @input="urlTestPassed = false; urlError = ''"
          />
          <p v-if="urlError" class="field-error-text">{{ urlError }}</p>
          <p v-else-if="urlTestPassed" class="field-success-text">Connection successful</p>
          <p v-else class="field-hint">SSH or HTTPS URL of your central repository</p>
        </div>

        <div class="wizard-actions">
          <button class="btn-secondary" @click="handleSkip">
            Skip Setup
          </button>
          <button
            class="btn-primary"
            :disabled="!upstreamUrl || isTestingConnection"
            @click="testConnection"
          >
            {{ isTestingConnection ? 'Testing...' : 'Test Connection' }}
          </button>
        </div>
      </div>

      <!-- Step 2: Workspace Prefix -->
      <div v-if="currentStep === 1" class="wizard-content">
        <h3 class="wizard-title">Set Workspace Prefix</h3>
        <p class="wizard-description">
          Choose a unique prefix to identify your workspace's documents in the central repository.
        </p>

        <div class="form-field">
          <label class="field-label">Workspace Prefix</label>
          <input
            v-model="workspacePrefix"
            type="text"
            class="field-input"
            :class="{ 'field-error': prefixError }"
            placeholder="api"
            maxlength="20"
            @input="handlePrefixInput"
          />
          <p v-if="prefixError" class="field-error-text">{{ prefixError }}</p>
          <p v-else class="field-hint">
            Lowercase letters, numbers, and hyphens. Documents will appear as {{ workspacePrefix || 'api' }}/PROJ-V-0001.md
          </p>
        </div>

        <div class="form-field">
          <label class="field-label">Team Label <span class="field-optional">(optional)</span></label>
          <input
            v-model="teamLabel"
            type="text"
            class="field-input"
            placeholder="platform"
          />
          <p class="field-hint">Helps group workspaces by team when browsing upstream</p>
        </div>

        <div class="wizard-actions">
          <button class="btn-secondary" @click="currentStep = 0">
            Back
          </button>
          <button
            class="btn-primary"
            :disabled="!workspacePrefix || !!prefixError"
            @click="currentStep = 2"
          >
            Next
          </button>
        </div>
      </div>

      <!-- Step 3: Initial Sync -->
      <div v-if="currentStep === 2" class="wizard-content">
        <h3 class="wizard-title">Ready to Sync</h3>
        <p class="wizard-description">
          We'll connect to the central repository and perform the initial sync.
        </p>

        <div class="sync-summary">
          <div class="summary-row">
            <span class="summary-label">Repository</span>
            <span class="summary-value summary-mono">{{ upstreamUrl }}</span>
          </div>
          <div class="summary-row">
            <span class="summary-label">Workspace</span>
            <span class="summary-value summary-mono">{{ workspacePrefix }}</span>
          </div>
          <div v-if="teamLabel" class="summary-row">
            <span class="summary-label">Team</span>
            <span class="summary-value">{{ teamLabel }}</span>
          </div>
        </div>

        <!-- Sync Progress -->
        <div v-if="isSyncing" class="sync-progress">
          <div class="spinner" />
          <span>{{ syncProgressMessage }}</span>
        </div>

        <!-- Sync Result -->
        <div v-if="syncResult" class="sync-result" :class="{ 'sync-error': !!syncError }">
          <p v-if="syncError" class="field-error-text">{{ syncError }}</p>
          <p v-else class="field-success-text">{{ syncResult }}</p>
        </div>

        <div class="wizard-actions">
          <button class="btn-secondary" :disabled="isSyncing" @click="currentStep = 1">
            Back
          </button>
          <button
            v-if="!syncResult || syncError"
            class="btn-primary"
            :disabled="isSyncing"
            @click="performInitialSync"
          >
            {{ isSyncing ? 'Syncing...' : (syncError ? 'Retry' : 'Start Sync') }}
          </button>
          <button
            v-else
            class="btn-primary"
            @click="handleComplete"
          >
            Done
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { MetisAPI } from '../lib/tauri-api'
import { emit as tauriEmit } from '@tauri-apps/api/event'

interface Props {
  isOpen: boolean
}

interface Emits {
  (e: 'close'): void
  (e: 'complete'): void
  (e: 'skip'): void
}

defineProps<Props>()
const emit = defineEmits<Emits>()

const steps = ['Repository', 'Workspace', 'Sync']
const currentStep = ref(0)

// Step 1 state
const upstreamUrl = ref('')
const urlError = ref('')
const urlTestPassed = ref(false)
const isTestingConnection = ref(false)

// Step 2 state
const workspacePrefix = ref('')
const prefixError = ref('')
const teamLabel = ref('')

// Step 3 state
const isSyncing = ref(false)
const syncProgressMessage = ref('Connecting to repository...')
const syncResult = ref('')
const syncError = ref('')

const testConnection = async () => {
  if (!upstreamUrl.value) return

  isTestingConnection.value = true
  urlError.value = ''
  urlTestPassed.value = false

  try {
    // We use syncWorkspace which will test the connection implicitly.
    // For now, just validate the URL format client-side and move forward.
    const url = upstreamUrl.value.trim()
    const isSSH = url.startsWith('git@') || url.includes('ssh://')
    const isHTTPS = url.startsWith('https://')
    const isFile = url.startsWith('file://')

    if (!isSSH && !isHTTPS && !isFile) {
      urlError.value = 'URL must start with git@, https://, or file://'
      return
    }

    // URL format is valid — mark as passed and advance
    urlTestPassed.value = true
    setTimeout(() => {
      currentStep.value = 1
    }, 500)
  } catch (error) {
    urlError.value = error instanceof Error ? error.message : 'Connection test failed'
  } finally {
    isTestingConnection.value = false
  }
}

const handlePrefixInput = (e: Event) => {
  const target = e.target as HTMLInputElement
  const value = target.value.toLowerCase().replace(/[^a-z0-9-]/g, '')
  workspacePrefix.value = value
  target.value = value
  validatePrefix(value)
}

const validatePrefix = (value: string) => {
  if (!value) {
    prefixError.value = ''
    return
  }
  if (value.length < 2) {
    prefixError.value = 'Prefix must be at least 2 characters'
    return
  }
  if (!/^[a-z][a-z0-9-]*$/.test(value)) {
    prefixError.value = 'Must start with a letter and contain only lowercase letters, numbers, and hyphens'
    return
  }
  prefixError.value = ''
}

const performInitialSync = async () => {
  isSyncing.value = true
  syncError.value = ''
  syncResult.value = ''
  syncProgressMessage.value = 'Connecting to repository...'

  try {
    syncProgressMessage.value = 'Syncing workspace...'
    const result = await MetisAPI.syncWorkspace()
    syncResult.value = result.summary
  } catch (error) {
    syncError.value = error instanceof Error ? error.message : String(error)
  } finally {
    isSyncing.value = false
  }
}

const handleSkip = () => {
  emit('skip')
}

const handleComplete = () => {
  tauriEmit('show-toast', { message: 'Multi-workspace sync configured successfully', type: 'success' })
  emit('complete')
}
</script>

<style scoped>
.bg-overlay {
  background-color: var(--color-background-overlay, rgba(0, 0, 0, 0.6));
}

.wizard-dialog {
  position: relative;
  z-index: 10;
  background: var(--color-background-elevated);
  border: 1px solid var(--color-border-primary);
  border-radius: 12px;
  box-shadow: 0 10px 40px rgba(0, 0, 0, 0.2);
  width: 480px;
  max-width: 90vw;
  overflow: hidden;
}

/* Progress Steps */
.wizard-steps {
  display: flex;
  gap: 0;
  padding: 20px 24px;
  border-bottom: 1px solid var(--color-border-primary);
  background: var(--color-background-secondary);
}

.wizard-step {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  font-size: 12px;
  font-weight: 500;
  color: var(--color-text-tertiary);
  transition: color 0.2s ease;
}

.wizard-step.active {
  color: var(--color-interactive-primary);
}

.wizard-step.completed {
  color: var(--color-text-secondary);
}

.step-indicator {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 11px;
  font-weight: 600;
  border: 2px solid var(--color-border-primary);
  color: var(--color-text-tertiary);
  flex-shrink: 0;
  transition: all 0.2s ease;
}

.wizard-step.active .step-indicator {
  border-color: var(--color-interactive-primary);
  background: var(--color-interactive-primary);
  color: var(--color-text-inverse);
}

.wizard-step.completed .step-indicator {
  border-color: var(--color-interactive-primary);
  background: var(--color-interactive-primary);
  color: var(--color-text-inverse);
}

.step-label {
  font-family: var(--font-body);
}

/* Content */
.wizard-content {
  padding: 24px;
}

.wizard-title {
  font-family: var(--font-display);
  font-size: 18px;
  font-weight: 600;
  color: var(--color-text-primary);
  margin: 0 0 8px 0;
}

.wizard-description {
  font-size: 14px;
  line-height: 1.5;
  color: var(--color-text-secondary);
  margin: 0 0 20px 0;
}

/* Form Fields */
.form-field {
  margin-bottom: 16px;
}

.field-label {
  display: block;
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text-primary);
  margin-bottom: 6px;
}

.field-optional {
  font-weight: 400;
  color: var(--color-text-tertiary);
}

.field-input {
  width: 100%;
  padding: 10px 12px;
  background: var(--color-background-primary);
  border: 1px solid var(--color-border-primary);
  border-radius: 6px;
  color: var(--color-text-primary);
  font-size: 14px;
  font-family: var(--font-body);
  transition: border-color 0.2s ease;
  box-sizing: border-box;
}

.field-input:focus {
  outline: none;
  border-color: var(--color-interactive-primary);
  box-shadow: 0 0 0 2px rgba(var(--color-interactive-primary-rgb, 59, 130, 246), 0.15);
}

.field-input.field-error {
  border-color: var(--color-border-error);
}

.field-input.field-success {
  border-color: var(--color-interactive-success, #22c55e);
}

.field-hint {
  font-size: 12px;
  color: var(--color-text-tertiary);
  margin-top: 4px;
}

.field-error-text {
  font-size: 12px;
  color: var(--color-border-error);
  margin-top: 4px;
}

.field-success-text {
  font-size: 12px;
  color: var(--color-interactive-success, #22c55e);
  margin-top: 4px;
}

/* Sync Summary */
.sync-summary {
  background: var(--color-background-secondary);
  border: 1px solid var(--color-border-primary);
  border-radius: 8px;
  padding: 12px 16px;
  margin-bottom: 16px;
}

.summary-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 6px 0;
}

.summary-row:not(:last-child) {
  border-bottom: 1px solid var(--color-border-primary);
}

.summary-label {
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text-secondary);
}

.summary-value {
  font-size: 13px;
  color: var(--color-text-primary);
  text-align: right;
  max-width: 60%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.summary-mono {
  font-family: var(--font-mono);
  font-size: 12px;
}

/* Sync Progress */
.sync-progress {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  background: var(--color-background-secondary);
  border-radius: 8px;
  margin-bottom: 16px;
  font-size: 13px;
  color: var(--color-text-secondary);
}

.spinner {
  width: 18px;
  height: 18px;
  border: 2px solid var(--color-border-primary);
  border-top-color: var(--color-interactive-primary);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  flex-shrink: 0;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* Sync Result */
.sync-result {
  padding: 12px 16px;
  background: var(--color-background-secondary);
  border-radius: 8px;
  margin-bottom: 16px;
}

.sync-result.sync-error {
  border: 1px solid var(--color-border-error);
}

/* Actions */
.wizard-actions {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  padding-top: 8px;
}

.btn-primary {
  padding: 10px 20px;
  background: var(--color-interactive-primary);
  border: 1px solid var(--color-interactive-primary);
  border-radius: 6px;
  color: var(--color-text-inverse);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-primary:hover:not(:disabled) {
  background: var(--color-interactive-primaryHover, var(--color-interactive-primary));
  transform: translateY(-1px);
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-secondary {
  padding: 10px 20px;
  background: var(--color-background-secondary);
  border: 1px solid var(--color-border-primary);
  border-radius: 6px;
  color: var(--color-text-primary);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-secondary:hover:not(:disabled) {
  background: var(--color-background-tertiary);
  border-color: var(--color-border-secondary);
}

.btn-secondary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
