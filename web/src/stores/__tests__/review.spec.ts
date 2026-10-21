import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'

import type { DueCard } from '@/generated/DueCard'
import { useReviewStore } from '../review'

function due(id: number): DueCard {
  return {
    card: { id, note_id: 1, kind: 'PictureToWord' },
    note: {
      id: 1,
      language: 'pt-BR',
      kind: {
        Word: { spelling: 'copo', ipa: null, gender: null, gloss: null, personal: null, image: 'abc', audio: null },
      },
    },
    state: { reps: 0, lapses: 0, stability: null, difficulty: null, last_review: null, due: null },
  }
}

function respond(...bodies: unknown[]) {
  const fetch = vi.fn<typeof globalThis.fetch>()
  for (const body of bodies) {
    fetch.mockResolvedValueOnce(new Response(JSON.stringify(body), { status: 200 }))
  }
  vi.stubGlobal('fetch', fetch)
  return fetch
}

describe('review session', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('moves to the next card after an answer and sends the grade with silent timing', async () => {
    const fetch = respond([due(1), due(2)], { reps: 1 })
    const review = useReviewStore()
    await review.load()
    expect(review.current?.card.id).toBe(1)

    review.reveal()
    await review.answer('Good')

    expect(review.current?.card.id).toBe(2)
    expect(review.revealed).toBe(false)
    const sent = JSON.parse(fetch.mock.calls[1]![1]!.body as string)
    expect(sent).toMatchObject({ card_id: 1, grade: 'Good', signals: { hinted: false } })
    expect(typeof sent.signals.response_ms).toBe('number')
  })

  it('keeps the card when saving the answer fails, so nothing is lost', async () => {
    const fetch = respond([due(1)])
    fetch.mockResolvedValueOnce(new Response(JSON.stringify({ error: 'boom' }), { status: 500 }))
    const review = useReviewStore()
    await review.load()

    await review.answer('Again')

    expect(review.current?.card.id).toBe(1)
    expect(review.error).toBe('boom')
  })
})
