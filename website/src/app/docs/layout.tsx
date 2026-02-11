import { Footer, Layout, Navbar } from 'nextra-theme-docs'
import { Banner, Head } from 'nextra/components'
import { getPageMap } from 'nextra/page-map'
import 'nextra-theme-docs/style.css'
import { Github } from 'lucide-react'
import Image from 'next/image'

export const metadata = {
  title: 'Mosaic Docs',
  description: 'Documentation for Mosaic - The Polytoria Package Manager'
}

const banner = <Banner storageKey="mosaic-beta">Mosaic is currently in beta. 🎉</Banner>

const navbar = (
  <Navbar
    logo={
      <div className="flex items-center gap-2">
        <Image src="/logo.png" alt="Mosaic" width={32} height={32} />
        <span className="font-bold text-lg">Mosaic</span>
      </div>
    }
    projectLink="https://github.com/doshibadev/mosaic"
  />
)

export default async function DocsLayout({ children }: { children: React.ReactNode }) {
  return (
    <div className="nextra-container">
      <Layout
        banner={banner}
        navbar={navbar}
        pageMap={await getPageMap()}
        docsRepositoryBase="https://github.com/doshibadev/mosaic/tree/main/website"
        sidebar={{ defaultOpen: true }}
      >
        {children}
      </Layout>
    </div>
  )
}
