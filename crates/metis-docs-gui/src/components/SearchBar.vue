<template>
  <div class="relative" ref="containerRef">
    <!-- Search Input -->
    <div class="search-input-wrapper">
      <input
        v-model="searchQuery"
        type="text"
        placeholder="Search documents..."
        class="search-input"
        @input="handleSearch"
        @focus="showResults = true"
        @keydown.escape="clearSearch"
        @keydown.enter="selectFirstResult"
        @keydown.down.prevent="navigateResults(1)"
        @keydown.up.prevent="navigateResults(-1)"
      />
      <!-- Search Icon -->
      <svg
        class="absolute top-1/2 -translate-y-1/2 text-secondary pointer-events-none"
        style="width: 20px; height: 20px; left: 16px;"
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
        />
      </svg>
      <!-- Clear Button -->
      <button
        v-if="searchQuery"
        @click="clearSearch"
        class="absolute top-1/2 -translate-y-1/2 text-secondary hover:text-primary"
        style="right: 12px;"
      >
        <svg style="width: 20px; height: 20px;" fill="currentColor" viewBox="0 0 20 20">
          <path
            fill-rule="evenodd"
            d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z"
            clip-rule="evenodd"
          />
        </svg>
      </button>
    </div>

    <!-- Search Results Dropdown -->
    <Transition name="dropdown">
      <div
        v-if="showResults && searchQuery && (results.length > 0 || isSearching || noResults)"
        class="search-dropdown"
      >
        <!-- Loading State -->
        <div v-if="isSearching" class="search-status">
          Searching...
        </div>

        <!-- No Results -->
        <div v-else-if="noResults" class="search-status">
          No documents found for "{{ searchQuery }}"
        </div>

        <!-- Results List -->
        <div v-else class="search-results">
          <button
            v-for="(doc, index) in results"
            :key="doc.id"
            @click="selectDocument(doc)"
            :class="['search-result-item', selectedIndex === index ? 'selected' : '']"
          >
            <div class="result-content">
              <!-- Document Type Badge -->
              <span
                :class="[
                  'type-badge',
                  getTypeBadgeClass(doc.document_type)
                ]"
              >
                {{ doc.short_code }}
              </span>
              <!-- Document Info -->
              <div class="result-info">
                <div class="result-title" :style="{ color: getDocTypeColor(doc.document_type) }">
                  {{ doc.title }}
                </div>
                <div class="result-meta">
                  <span>{{ doc.document_type }}</span>
                  <span class="meta-separator">|</span>
                  <span>{{ doc.phase }}</span>
                </div>
              </div>
            </div>
          </button>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from 'vue'
import { searchDocuments, type DocumentInfo } from '../lib/tauri-api'
import { emit } from '@tauri-apps/api/event'

const emit_event = emit

const searchQuery = ref('')
const results = ref<DocumentInfo[]>([])
const showResults = ref(false)
const isSearching = ref(false)
const noResults = ref(false)
const selectedIndex = ref(-1)
const containerRef = ref<HTMLElement | null>(null)

let searchTimeout: ReturnType<typeof setTimeout> | null = null

// Debounced search
const handleSearch = () => {
  if (searchTimeout) clearTimeout(searchTimeout)

  if (!searchQuery.value.trim()) {
    results.value = []
    noResults.value = false
    return
  }

  isSearching.value = true
  noResults.value = false
  selectedIndex.value = -1

  searchTimeout = setTimeout(async () => {
    try {
      const searchResults = await searchDocuments(searchQuery.value)
      results.value = searchResults
      noResults.value = searchResults.length === 0
    } catch (error) {
      console.error('Search failed:', error)
      emit_event('show-toast', { message: 'Search failed', type: 'error' })
      results.value = []
      noResults.value = true
    } finally {
      isSearching.value = false
    }
  }, 300) // 300ms debounce
}

const clearSearch = () => {
  searchQuery.value = ''
  results.value = []
  showResults.value = false
  noResults.value = false
  selectedIndex.value = -1
}

const selectDocument = (doc: DocumentInfo) => {
  // Emit event for parent to handle navigation
  emit_event('search-select-document', doc)
  clearSearch()
}

const selectFirstResult = () => {
  if (results.value.length > 0) {
    const index = selectedIndex.value >= 0 ? selectedIndex.value : 0
    selectDocument(results.value[index])
  }
}

const navigateResults = (direction: number) => {
  if (results.value.length === 0) return

  selectedIndex.value += direction
  if (selectedIndex.value < 0) selectedIndex.value = results.value.length - 1
  if (selectedIndex.value >= results.value.length) selectedIndex.value = 0
}

const getTypeBadgeClass = (type: string): string => {
  switch (type) {
    case 'vision':
      return 'badge-vision'
    case 'strategy':
      return 'badge-strategy'
    case 'initiative':
      return 'badge-initiative'
    case 'task':
      return 'badge-task'
    case 'adr':
      return 'badge-adr'
    default:
      return 'badge-default'
  }
}

const getDocTypeColor = (type: string): string => {
  // Return CSS variable for the document type color
  switch (type) {
    case 'vision':
      return 'var(--color-documentType-vision)'
    case 'strategy':
      return 'var(--color-documentType-strategy)'
    case 'initiative':
      return 'var(--color-documentType-initiative)'
    case 'task':
      return 'var(--color-documentType-task)'
    case 'adr':
      return 'var(--color-documentType-adr)'
    default:
      return 'var(--color-text-primary)'
  }
}

// Click outside to close
const handleClickOutside = (event: MouseEvent) => {
  if (containerRef.value && !containerRef.value.contains(event.target as Node)) {
    showResults.value = false
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
  if (searchTimeout) clearTimeout(searchTimeout)
})

// Reset selected index when results change
watch(results, () => {
  selectedIndex.value = -1
})
</script>

<style scoped>
/* Search input wrapper with focus effects */
.search-input-wrapper {
  position: relative;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  border-radius: 10px;
}

.search-input-wrapper:focus-within {
  transform: scale(1.02);
}

.search-input {
  width: 100%;
  padding: 12px 24px 12px 48px;
  font-family: var(--font-body);
  font-size: 15px;
  background: var(--color-background-elevated);
  color: var(--color-text-primary);
  border: 1px solid var(--color-border-primary);
  border-radius: 10px;
  min-width: 200px;
  outline: none;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.search-input::placeholder {
  color: var(--color-text-tertiary);
}

.search-input:focus {
  border-color: var(--color-interactive-primary);
  box-shadow:
    0 0 0 3px color-mix(in srgb, var(--color-interactive-primary) 20%, transparent),
    0 8px 24px -8px rgba(0, 0, 0, 0.15);
}

/* Dropdown container */
.search-dropdown {
  position: absolute;
  top: 100%;
  left: 0;
  margin-top: 8px;
  background-color: var(--color-background-elevated);
  border: 1px solid var(--color-border-primary);
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  max-height: 384px;
  overflow-y: auto;
  z-index: 50;
  min-width: 400px;
  width: max-content;
  max-width: 600px;
}

/* Loading/No results status */
.search-status {
  padding: 16px;
  text-align: center;
  color: var(--color-text-secondary);
  font-size: 14px;
}

/* Results list */
.search-results {
  padding: 8px 0;
}

/* Individual result item */
.search-result-item {
  width: 100%;
  padding: 12px 16px;
  text-align: left;
  background: transparent;
  border: none;
  cursor: pointer;
  transition: background-color 0.2s ease;
}

.search-result-item:hover,
.search-result-item.selected {
  background-color: var(--color-background-secondary);
}

.result-content {
  display: flex;
  align-items: center;
  gap: 12px;
}

/* Type badge */
.type-badge {
  font-family: var(--font-mono);
  padding: 3px 8px;
  font-size: 10px;
  font-weight: 600;
  border-radius: 4px;
  flex-shrink: 0;
  letter-spacing: 0.02em;
}

.badge-vision {
  background-color: color-mix(in srgb, var(--color-documentType-vision) 20%, transparent);
  color: var(--color-documentType-vision);
}

.badge-strategy {
  background-color: color-mix(in srgb, var(--color-documentType-strategy) 20%, transparent);
  color: var(--color-documentType-strategy);
}

.badge-initiative {
  background-color: color-mix(in srgb, var(--color-documentType-initiative) 20%, transparent);
  color: var(--color-documentType-initiative);
}

.badge-task {
  background-color: color-mix(in srgb, var(--color-documentType-task) 20%, transparent);
  color: var(--color-documentType-task);
}

.badge-adr {
  background-color: color-mix(in srgb, var(--color-documentType-adr) 20%, transparent);
  color: var(--color-documentType-adr);
}

.badge-default {
  background-color: var(--color-background-tertiary);
  color: var(--color-text-secondary);
}

/* Result info */
.result-info {
  flex: 1;
  min-width: 0;
}

.result-title {
  font-size: 14px;
  font-weight: 500;
  color: var(--color-text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.result-meta {
  font-size: 12px;
  color: var(--color-text-secondary);
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 2px;
}

.meta-separator {
  opacity: 0.5;
}

/* Dropdown transitions */
.dropdown-enter-active,
.dropdown-leave-active {
  transition: all 0.2s ease;
}

.dropdown-enter-from,
.dropdown-leave-to {
  opacity: 0;
  transform: translateY(-10px);
}
</style>
