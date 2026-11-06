import { defineStore } from 'pinia'
import { ref } from 'vue'

export const usePreferencesStore = defineStore('preferences', () => {
  const mobileNavigationOpen = ref(false)
  const periodDays = ref(7)

  function reset() {
    mobileNavigationOpen.value = false
    periodDays.value = 7
  }

  return { mobileNavigationOpen, periodDays, reset }
})
