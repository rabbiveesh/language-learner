import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'

import { useSessionStore } from '../session'

vi.mock('@simplewebauthn/browser', () => ({
  startRegistration: async () => ({ id: 'new-passkey' }),
  startAuthentication: async () => {
    throw new Error('The operation either timed out or was not allowed.')
  },
}))

function respond(...bodies: unknown[]) {
  const fetch = vi.fn<typeof globalThis.fetch>()
  for (const body of bodies) {
    fetch.mockResolvedValueOnce(new Response(JSON.stringify(body), { status: 200 }))
  }
  vi.stubGlobal('fetch', fetch)
  return fetch
}

describe('session', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('creates the first passkey with the setup token and ends up signed in', async () => {
    const fetch = respond({ publicKey: { challenge: 'c' } }, { signed_in: true, needs_setup: false })
    const session = useSessionStore()

    expect(await session.register('let-me-in')).toBe(true)

    expect(session.status?.signed_in).toBe(true)
    expect(JSON.parse(fetch.mock.calls[0]![1]!.body as string)).toEqual({ setup_token: 'let-me-in' })
    expect(JSON.parse(fetch.mock.calls[1]![1]!.body as string)).toEqual({ id: 'new-passkey' })
  })

  it('a cancelled passkey prompt leaves you signed out with a message', async () => {
    respond({ publicKey: { challenge: 'c' } })
    const session = useSessionStore()

    expect(await session.signIn()).toBe(false)

    expect(session.status).toBeNull()
    expect(session.error).toMatch(/not allowed/)
    expect(session.busy).toBe(false)
  })
})
