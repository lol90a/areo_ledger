'use client'

import { createContext, useContext, useEffect } from 'react'

type Theme = 'dark'

interface ThemeContextType {
  theme: Theme
  toggleTheme: () => void
}

const ThemeContext = createContext<ThemeContextType | undefined>(undefined)

export function ThemeProvider({ children }: { children: React.ReactNode }) {
  useEffect(() => {
    document.documentElement.classList.remove('light')
    document.documentElement.classList.add('dark')
    localStorage.setItem('theme', 'dark')
  }, [])

  const value: ThemeContextType = {
    theme: 'dark',
    toggleTheme: () => {},
  }

  return (
    <ThemeContext.Provider value={value}>
      {children}
    </ThemeContext.Provider>
  )
}

export function useTheme() {
  const context = useContext(ThemeContext)
  if (!context) throw new Error('useTheme must be used within ThemeProvider')
  return context
}