import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import { api } from '@/api'
import type { DueCard } from '@/generated/DueCard'
import type { Grade } from '@/generated/Grade'

/**
 * The review session: a queue of due cards, the one being shown, and whether its answer is
 * revealed. Response time is measured silently and sent as a signal, never shown: there is no
 * time pressure in this app.
 */
export const useReviewStore = defineStore('review', () => {
  const queue = ref<DueCard[]>([])
  const revealed = ref(false)
  const loading = ref(false)
  const error = ref<string | null>(null)
  let shownAt = 0

  const current = computed(() => queue.value[0] ?? null)

  async function load() {
    loading.value = true
    error.value = null
    try {
      queue.value = await api.due()
      revealed.value = false
      shownAt = performance.now()
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      loading.value = false
    }
  }

  function reveal() {
    revealed.value = true
  }

  // TODO(design): typed answers (picture→word, audio→word) graded on the server with
  // `check_typed`, which suggests the grade; for now every card is self-graded.
  async function answer(grade: Grade, typed: string | null = null) {
    const card = current.value
    if (!card) return
    const response_ms = Math.round(performance.now() - shownAt)
    try {
      await api.review({ card_id: card.card.id, grade, signals: { response_ms, answer: typed, hinted: false } })
    } catch (e) {
      // Keep the card so the answer can be retried.
      error.value = e instanceof Error ? e.message : String(e)
      return
    }
    queue.value = queue.value.slice(1)
    revealed.value = false
    shownAt = performance.now()
  }

  return { queue, current, revealed, loading, error, load, reveal, answer }
})
