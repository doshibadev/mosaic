import { notFound } from "next/navigation";
import { getDocBySlug, getAllDocSlugs } from "@/lib/mdx";
import Link from "next/link";
import { cn } from "@/lib/utils";

interface DocPageProps {
  params: Promise<{
    slug: string[];
  }>;
}

export async function generateStaticParams() {
  const slugs = await getAllDocSlugs();
  return slugs.map((slug) => ({
    slug,
  }));
}

export default async function DocPage({ params }: DocPageProps) {
  const { slug } = await params;
  const slugjoin = slug.join("/");
  const doc = await getDocBySlug(slug);

  if (!doc) {
    notFound();
  }

  const sidebarLinks = [
    { section: "Getting Started", items: [
      { name: "Introduction", href: "/docs/getting-started" },
      { name: "CLI Reference", href: "/docs/cli-reference" },
    ]},
    { section: "Guides", items: [
      { name: "Publishing", href: "/docs/publishing" },
    ]},
  ];

  return (
    <div className="mx-auto max-w-6xl px-6 py-12 lg:flex lg:gap-16 min-h-screen">
      {/* Sidebar */}
      <aside className="hidden lg:block w-52 shrink-0">
        <nav className="sticky top-20 space-y-6">
          {sidebarLinks.map((group) => (
            <div key={group.section}>
              <h4 className="text-sm font-medium text-muted-foreground/50 uppercase tracking-wider mb-3">
                {group.section}
              </h4>
              <div className="space-y-1">
                {group.items.map((item) => (
                  <Link
                    key={item.href}
                    href={item.href}
                    className={cn(
                      "block px-3 py-2 rounded-lg text-base transition-colors",
                      slugjoin === item.href.replace("/docs/", "")
                        ? "bg-accent text-primary font-medium"
                        : "text-muted-foreground hover:text-foreground hover:bg-accent"
                    )}
                  >
                    {item.name}
                  </Link>
                ))}
              </div>
            </div>
          ))}
        </nav>
      </aside>

      {/* Content */}
      <article className="min-w-0 flex-1">
        <header className="mb-10 pb-6 border-b border-border/50">
          <h1 className="text-3xl font-bold text-foreground mb-2">
            {doc.frontmatter.title}
          </h1>
          <p className="text-lg text-muted-foreground">
            {doc.frontmatter.description}
          </p>
        </header>

        <div className="prose prose-invert prose-lg max-w-none
          prose-headings:font-semibold prose-headings:text-foreground
          prose-p:text-muted-foreground prose-p:leading-relaxed
          prose-a:text-secondary prose-a:no-underline hover:prose-a:underline
          prose-pre:bg-card prose-pre:border prose-pre:border-border prose-pre:rounded-lg
          prose-code:text-primary prose-code:bg-accent prose-code:rounded prose-code:px-1.5 prose-code:py-0.5 prose-code:before:content-none prose-code:after:content-none prose-code:font-mono
          prose-strong:text-foreground
          prose-li:text-muted-foreground
        ">
          {doc.content}
        </div>
      </article>
    </div>
  );
}
