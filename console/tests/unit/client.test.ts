import { afterEach, describe, expect, it, vi } from 'vitest'
import { request, type ApiError } from '../../app/lib/api/client'

afterEach(() => vi.unstubAllGlobals())

describe('same-origin session transport', () => {
  it('rejects external and protocol-relative URLs before issuing a request', async () => {
    const fetch = vi.fn()
    vi.stubGlobal('fetch', fetch)
    await expect(request('https://example.com')).rejects.toThrow('same-origin')
    await expect(request('//example.com')).rejects.toThrow('same-origin')
    await expect(request('/\\example.com')).rejects.toThrow('same-origin')
    expect(fetch).not.toHaveBeenCalled()
  })

  it('uses cookies, disables cache, and never adds bearer credentials', async () => {
    const fetch = vi.fn().mockResolvedValue(new Response('{"username":"admin"}'))
    vi.stubGlobal('fetch', fetch)
    await request('/auth/session')
    const options = fetch.mock.calls[0]?.[1]
    expect(options.credentials).toBe('same-origin')
    expect(options.cache).toBe('no-store')
    expect(options.headers).toBeUndefined()
  })

  it('preserves the HTTP status when an API error body cannot be decoded', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(new Response('Unauthorized', { status: 401 })))
    await expect(request('/admin/state')).rejects.toMatchObject<ApiError>({ status: 401 })
  })
})
