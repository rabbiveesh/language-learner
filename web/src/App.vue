<script setup lang="ts">
import { RouterView } from 'vue-router'

import { useSessionStore } from '@/stores/session'

const session = useSessionStore()

const links = [
  { label: 'Review', to: '/' },
  { label: 'Words', to: '/words' },
  { label: 'Grammar', to: '/grammar' },
  { label: 'Account', to: '/account' },
]
</script>

<template>
  <UApp>
    <UContainer class="py-4">
      <nav v-if="session.status?.signed_in" class="mb-6 flex gap-2">
        <UButton v-for="link in links" :key="link.to" :to="link.to" variant="ghost" :label="link.label" />
      </nav>
      <RouterView />
    </UContainer>
  </UApp>
</template>
