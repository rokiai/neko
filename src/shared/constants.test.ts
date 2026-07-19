import { describe, expect, it } from 'vitest'
import { APP_ID, APP_NAME } from './constants'

describe('shared/constants', () => {
  it('exposes stable product identity', () => {
    expect(APP_NAME).toBe('Neko')
    expect(APP_ID).toBe('com.neko.app')
  })
})
