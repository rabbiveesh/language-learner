// Calls to the server. The types are generated from the Rust code (`cargo test` writes them to
// src/generated); never edit those by hand.
import type {
  AuthenticationResponseJSON,
  PublicKeyCredentialCreationOptionsJSON,
  PublicKeyCredentialRequestOptionsJSON,
  RegistrationResponseJSON,
} from '@simplewebauthn/browser'

import type { AuthStatus } from './generated/AuthStatus'
import type { CardState } from './generated/CardState'
import type { DueCard } from './generated/DueCard'
import type { NewReview } from './generated/NewReview'
import type { NewWordNote } from './generated/NewWordNote'
import type { NoteWithCards } from './generated/NoteWithCards'
import type { RegisterStart } from './generated/RegisterStart'

export class ApiError extends Error {
  constructor(
    readonly status: number,
    message: string,
  ) {
    super(message)
  }
}

async function call<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`/api${path}`, {
    ...init,
    headers: { 'content-type': 'application/json', ...init?.headers },
  })
  if (!res.ok) {
    const body = await res.json().catch(() => ({}))
    throw new ApiError(res.status, body.error ?? res.statusText)
  }
  return res.json()
}

const post = (body?: unknown): RequestInit => ({
  method: 'POST',
  body: body === undefined ? undefined : JSON.stringify(body),
})

/** The server's WebAuthn challenges wrap the options in `publicKey`. */
type Challenge<T> = { publicKey: T }

export const auth = {
  status: () => call<AuthStatus>('/auth/status'),
  registerStart: (r: RegisterStart) =>
    call<Challenge<PublicKeyCredentialCreationOptionsJSON>>('/auth/register/start', post(r)),
  registerFinish: (c: RegistrationResponseJSON) => call<AuthStatus>('/auth/register/finish', post(c)),
  signInStart: () => call<Challenge<PublicKeyCredentialRequestOptionsJSON>>('/auth/sign-in/start', post()),
  signInFinish: (c: AuthenticationResponseJSON) => call<AuthStatus>('/auth/sign-in/finish', post(c)),
  signOut: () => call<AuthStatus>('/auth/sign-out', post()),
}

export const api = {
  due: () => call<DueCard[]>('/due'),
  review: (r: NewReview) => call<CardState>('/reviews', { method: 'POST', body: JSON.stringify(r) }),
  addWord: (n: NewWordNote) => call<NoteWithCards>('/notes', { method: 'POST', body: JSON.stringify(n) }),
}
