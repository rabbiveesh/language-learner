// Calls to the server. The types are generated from the Rust code (`cargo test` writes them to
// src/generated); never edit those by hand.
import type { CardState } from './generated/CardState'
import type { DueCard } from './generated/DueCard'
import type { NewReview } from './generated/NewReview'
import type { NewWordNote } from './generated/NewWordNote'
import type { NoteWithCards } from './generated/NoteWithCards'

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

export const api = {
  due: () => call<DueCard[]>('/due'),
  review: (r: NewReview) => call<CardState>('/reviews', { method: 'POST', body: JSON.stringify(r) }),
  addWord: (n: NewWordNote) => call<NoteWithCards>('/notes', { method: 'POST', body: JSON.stringify(n) }),
}
