<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import { useSessionStore } from '@/stores/session'

const session = useSessionStore()
const route = useRoute()
const router = useRouter()

const setup = computed(() => session.status?.needs_setup ?? false)
const token = ref(typeof route.query.token === 'string' ? route.query.token : '')

async function go() {
  const ok = setup.value ? await session.register(token.value || null) : await session.signIn()
  if (ok) {
    await router.replace(typeof route.query.next === 'string' ? route.query.next : '/')
  }
}
</script>

<template>
  <section class="mx-auto mt-16 max-w-sm">
    <UCard>
      <template #header>
        <h1 class="text-xl">{{ setup ? 'Set up Fala' : 'Fala' }}</h1>
      </template>

      <div class="space-y-4">
        <template v-if="setup">
          <p>Create a passkey on this device. You'll use it to sign in from now on.</p>
          <UInput v-if="!route.query.token" v-model="token" placeholder="Setup token" class="w-full" />
        </template>

        <UAlert v-if="session.error" color="error" :title="session.error" />

        <UButton
          block
          :loading="session.busy"
          :label="setup ? 'Create passkey' : 'Sign in with passkey'"
          @click="go"
        />
      </div>
    </UCard>
  </section>
</template>
