import { startAuthentication, startRegistration } from '@simplewebauthn/browser'
import { defineStore } from 'pinia'
import { ref } from 'vue'

import { auth } from '@/api'
import type { AuthStatus } from '@/generated/AuthStatus'

/** Whether the learner is signed in, and the passkey ceremonies that change it. */
export const useSessionStore = defineStore('session', () => {
  const status = ref<AuthStatus | null>(null)
  const error = ref<string | null>(null)
  const busy = ref(false)

  async function load() {
    status.value = await auth.status()
    return status.value
  }

  async function run(ceremony: () => Promise<AuthStatus>) {
    busy.value = true
    error.value = null
    try {
      status.value = await ceremony()
      return true
    } catch (e) {
      // A cancelled browser prompt lands here too; it's not worth more than a line.
      error.value = e instanceof Error ? e.message : String(e)
      return false
    } finally {
      busy.value = false
    }
  }

  /** Add a passkey on this device: the first one needs the setup token. */
  const register = (setupToken: string | null = null) =>
    run(async () => {
      const { publicKey } = await auth.registerStart({ setup_token: setupToken })
      return auth.registerFinish(await startRegistration({ optionsJSON: publicKey }))
    })

  const signIn = () =>
    run(async () => {
      const { publicKey } = await auth.signInStart()
      return auth.signInFinish(await startAuthentication({ optionsJSON: publicKey }))
    })

  const signOut = () => run(() => auth.signOut())

  return { status, error, busy, load, register, signIn, signOut }
})
