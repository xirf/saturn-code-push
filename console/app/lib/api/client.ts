export class ApiError extends Error {
  constructor(
    message: string,
    public readonly status: number,
  ) {
    super(message)
    this.name = 'ApiError'
  }
}

interface RequestOptions {
  method?: 'GET' | 'POST' | 'PATCH'
  body?: unknown
  signal?: AbortSignal
}

/** All console requests use the existing same-origin HTTP-only session cookie. */
export async function request(path: string, options: RequestOptions = {}): Promise<unknown> {
  const origin = typeof location === 'undefined' ? 'http://localhost' : location.origin
  if (!path.startsWith('/') || new URL(path, origin).origin !== origin) {
    throw new Error('API requests must use a same-origin path')
  }

  const response = await fetch(path, {
    method: options.method ?? 'GET',
    credentials: 'same-origin',
    cache: 'no-store',
    signal: options.signal,
    ...(options.body === undefined
      ? {}
      : {
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(options.body),
        }),
  })
  const payload: unknown = await response.json().catch(() => null)

  if (!response.ok) {
    const message =
      payload &&
      typeof payload === 'object' &&
      'message' in payload &&
      typeof payload.message === 'string'
        ? payload.message
        : `Request failed (${response.status})`
    throw new ApiError(message, response.status)
  }
  return payload
}

export function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : 'Something went wrong. Please try again.'
}
