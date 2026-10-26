import { z } from 'zod'

// Configure before constructing any schema. Strict CSP also reports a caught
// code-generation probe, so validation must never attempt that probe.
z.config({ jitless: true })

export { z }
