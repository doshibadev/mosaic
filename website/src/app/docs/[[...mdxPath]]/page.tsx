import { importPage } from 'nextra/pages'
import { useMDXComponents } from '../../../mdx-components'

export async function generateStaticParams() {
  // In dynamic mode, we don't need to generate params upfront for dev, 
  // but for production build, Nextra handles this. 
  // For catch-all, we return an empty array or specific paths.
  return []
}

export async function generateMetadata(props: any) {
  const params = await props.params
  const { metadata } = await importPage(params.mdxPath)
  return metadata
}

const { wrapper: Wrapper, ...components } = useMDXComponents({})

export default async function Page(props: any) {
  const params = await props.params
  const result = await importPage(params.mdxPath)
  const { default: MDXContent, toc, metadata, sourceCode } = result
  
  return (
    <Wrapper toc={toc} metadata={metadata} sourceCode={sourceCode}>
      <MDXContent {...props} params={params} />
    </Wrapper>
  )
}
