import { useEffect } from "react"

export function useSystemColorScheme() {
  useEffect(() => {
    const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)")
    const root = document.documentElement

    function applySystemColorScheme() {
      const prefersDark = mediaQuery.matches

      root.classList.toggle("dark", prefersDark)
      root.style.colorScheme = prefersDark ? "dark" : "light"
    }

    applySystemColorScheme()
    mediaQuery.addEventListener("change", applySystemColorScheme)

    return () => {
      mediaQuery.removeEventListener("change", applySystemColorScheme)
      root.classList.remove("dark")
      root.style.colorScheme = ""
    }
  }, [])
}
