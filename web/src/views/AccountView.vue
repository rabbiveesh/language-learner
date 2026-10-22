<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'

import { useSessionStore } from '@/stores/session'

const session = useSessionStore()
const router = useRouter()
const added = ref(false)

async function addDevice() {
  added.value = await session.register()
}

async function signOut() {
  if (await session.signOut()) {
    await router.replace({ name: 'sign-in' })
  }
}
</script>

<template>
  <section class="mx-auto max-w-xl space-y-4">
    <UCard>
      <template #header>Passkeys</template>
      <div class="space-y-3">
        <p>Add a passkey on this device so you can sign in here too.</p>
        <UAlert v-if="added" color="success" title="Passkey added." />
        <UAlert v-if="session.error" color="error" :title="session.error" />
        <UButton :loading="session.busy" label="Add a passkey on this device" @click="addDevice" />
      </div>
    </UCard>
    <UButton variant="ghost" color="neutral" label="Sign out" @click="signOut" />
  </section>
</template>
