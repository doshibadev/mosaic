import React from 'react'

const config = {
  logo: (
    <div className="flex items-center gap-2">
      <img src="/logo.png" alt="Mosaic" width={24} height={24} />
      <span className="font-bold text-lg">mosaic</span>
    </div>
  ),
  project: {
    link: 'https://github.com/doshibadev/mosaic',
  },
  docsRepositoryBase: 'https://github.com/doshibadev/mosaic/tree/main/website',
  footer: {
    text: `Mosaic — Polytoria Package Manager © ${new Date().getFullYear()}`,
  },
  darkMode: true,
  nextThemes: {
    defaultTheme: 'dark'
  }
}

export default config
