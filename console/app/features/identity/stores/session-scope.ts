import { defineStore } from 'pinia'
import { ref } from 'vue'

/** A cache boundary, not a second copy of the authenticated user. */
export const useSessionScope = defineStore('session-scope', () => {
  const generation = ref(0)
  function advance() {
    generation.value += 1
  }
  return { generation, advance }
})
