<script setup lang="ts">
import { computed, onMounted } from 'vue'

import type { Grade } from '@/generated/Grade'
import { useReviewStore } from '@/stores/review'

const review = useReviewStore()
onMounted(review.load)

const word = computed(() => review.current?.note.kind.Word ?? null)
const kind = computed(() => review.current?.card.kind ?? null)

const grades: { grade: Grade; label: string; color: 'error' | 'warning' | 'primary' | 'success' }[] = [
  { grade: 'Again', label: 'Again', color: 'error' },
  { grade: 'Hard', label: 'Hard', color: 'warning' },
  { grade: 'Good', label: 'Good', color: 'primary' },
  { grade: 'Easy', label: 'Easy', color: 'success' },
]

// TODO(design): media is served by content hash once the server stores files; until then the
// picture and audio refs are shown as text.
</script>

<template>
  <section class="mx-auto max-w-xl">
    <UAlert v-if="review.error" color="error" :title="review.error" class="mb-4" />

    <p v-if="review.loading">Loading…</p>

    <UCard v-else-if="review.current && word">
      <template #header>
        <span class="text-sm text-muted">{{ kind }}</span>
      </template>

      <div class="space-y-4 text-center">
        <div v-if="kind === 'PictureToWord' || kind === 'Speaking'" class="text-muted">
          picture {{ word.image }}
        </div>
        <div v-else-if="kind === 'AudioToWord'" class="text-muted">audio {{ word.audio }}</div>
        <div v-else class="text-3xl">{{ word.spelling }}</div>

        <p v-if="kind === 'Speaking'">Say it out loud.</p>

        <div v-if="review.revealed" class="space-y-1">
          <div class="text-3xl">{{ word.spelling }}</div>
          <div v-if="word.ipa" class="text-muted">/{{ word.ipa }}/</div>
          <div v-if="kind === 'WordToPicture'" class="text-muted">picture {{ word.image }}</div>
        </div>
      </div>

      <template #footer>
        <div v-if="!review.revealed" class="flex justify-center">
          <UButton label="Show answer" @click="review.reveal" />
        </div>
        <div v-else class="flex justify-center gap-2">
          <UButton
            v-for="g in grades"
            :key="g.grade"
            :label="g.label"
            :color="g.color"
            variant="soft"
            @click="review.answer(g.grade)"
          />
        </div>
      </template>
    </UCard>

    <UCard v-else>
      <p>Nothing due right now. Nice.</p>
    </UCard>
  </section>
</template>
