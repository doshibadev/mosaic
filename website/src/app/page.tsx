import Link from "next/link";
import Image from "next/image";
import { ArrowRight, Github, Terminal } from "lucide-react";
import { searchPackages } from "@/lib/registry";
import { Suspense } from "react";

export default function HomePage() {
  return (
    <div className="flex flex-col min-h-screen">
      {/* Hero — centered, logo-forward */}
      <section className="flex-1 flex items-center justify-center px-6 py-20 md:py-28">
        <div className="text-center w-full max-w-5xl">
          <Image
            src="/logo.png"
            alt="Mosaic"
            width={96}
            height={96}
            className="mx-auto mb-8"
          />

          <h1 className="text-5xl md:text-6xl font-bold text-foreground tracking-tight mb-5">
            mosaic
          </h1>

          <p className="text-2xl text-muted-foreground mb-10 leading-relaxed">
            A package manager for{" "}
            <Link href="https://polytoria.com" target="_blank" className="text-secondary hover:underline">
              Polytoria
            </Link>
          </p>

          {/* Install command — the main thing */}
          <div className="bg-card border border-border rounded-lg p-6 mb-10 max-w-xl mx-auto text-left">
            <div className="flex items-center gap-2 text-muted-foreground/50 text-base mb-4">
              <Terminal className="h-5 w-5" />
              <span>Get started</span>
            </div>
            <div className="font-mono text-lg space-y-2">
              <div>
                <span className="text-muted-foreground/50 select-none">$ </span>
                <span className="text-foreground">cargo install mosaic-cli</span>
              </div>
              <div>
                <span className="text-muted-foreground/50 select-none">$ </span>
                <span className="text-foreground">mosaic init</span>
              </div>
            </div>
          </div>

          <div className="flex gap-4 justify-center flex-wrap">
            <Link
              href="/docs/getting-started"
              className="inline-flex h-12 items-center gap-2 rounded-lg bg-primary text-primary-foreground px-7 text-base font-medium hover:bg-primary/90 transition-colors"
            >
              Get Started
              <ArrowRight className="h-4 w-4" />
            </Link>
            <Link
              href="https://github.com/doshibadev/mosaic"
              target="_blank"
              className="inline-flex h-12 items-center gap-2 rounded-lg border border-border text-muted-foreground px-7 text-base font-medium hover:text-foreground hover:border-foreground/20 transition-colors"
            >
              <Github className="h-5 w-5" />
              GitHub
            </Link>
          </div>
        </div>
      </section>

      {/* Featured Packages (auto-hides if empty) */}
      <Suspense fallback={null}>
        <FeaturedPackageList />
      </Suspense>

      {/* About */}
      <section className="border-t border-border/50">
        <div className="mx-auto max-w-4xl px-6 py-20 md:py-24 text-center">
          <h2 className="text-3xl font-semibold text-foreground mb-8">What is Mosaic?</h2>
          <div className="text-lg md:text-xl text-muted-foreground leading-relaxed space-y-5">
            <p>
              Mosaic is a package manager for{" "}
              <Link href="https://polytoria.com" target="_blank" className="text-secondary hover:underline">Polytoria</Link>,
              inspired by <Link href="https://crates.io" target="_blank" className="text-secondary hover:underline">Cargo</Link> and{" "}
              <Link href="https://www.npmjs.com" target="_blank" className="text-secondary hover:underline">npm</Link>.
              It lets you share and reuse Lua code across Polytoria projects.
            </p>
            <p>
              There are two parts: the <code className="text-primary bg-card px-2 py-1 rounded text-[17px] font-mono border border-border">mosaic</code> CLI
              and a central registry. The CLI is all you need to install packages, manage dependencies, and publish your own.
            </p>
            <p>
              It&apos;s written in Rust, open source, and still early — but it works. If you build things on Polytoria, give it a try.
            </p>
          </div>
        </div>
      </section>

      {/* Quick reference */}
      <section className="border-t border-border/50">
        <div className="mx-auto max-w-4xl px-6 py-20 md:py-24">
          <h2 className="text-3xl font-semibold text-foreground mb-8 text-center">Quick reference</h2>
          <div className="bg-card border border-border rounded-lg overflow-hidden">
            <table className="w-full text-left">
              <tbody className="divide-y divide-border">
                {[
                  ["mosaic init", "Create a new project"],
                  ["mosaic add <package>", "Add a dependency"],
                  ["mosaic remove <package>", "Remove a dependency"],
                  ["mosaic install", "Install from lockfile"],
                  ["mosaic search <query>", "Search the registry"],
                  ["mosaic publish", "Publish your package"],
                ].map(([cmd, desc]) => (
                  <tr key={cmd} className="hover:bg-accent/50 transition-colors">
                    <td className="px-6 py-4 font-mono text-base text-foreground whitespace-nowrap">{cmd}</td>
                    <td className="px-6 py-4 text-base text-muted-foreground">{desc}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      </section>

      {/* Footer */}
      <footer className="border-t border-border/50 mt-auto">
        <div className="mx-auto max-w-4xl px-6 py-10 flex items-center justify-between">
          <p className="text-sm text-muted-foreground/60">
            © 2026 Mosaic · MIT License
          </p>
          <div className="flex items-center gap-6 text-sm text-muted-foreground/60">
            <Link href="/docs" className="hover:text-foreground transition-colors">Docs</Link>
            <Link href="/packages" className="hover:text-foreground transition-colors">Packages</Link>
            <Link href="https://github.com/doshibadev/mosaic" target="_blank" className="hover:text-foreground transition-colors">
              <Github className="h-4 w-4" />
            </Link>
          </div>
        </div>
      </footer>
    </div>
  );
}

async function FeaturedPackageList() {
  const packages = await searchPackages("");
  const featured = packages.slice(0, 3);

  if (featured.length === 0) {
    return null;
  }

  return (
    <section className="border-t border-border/50 bg-card/30">
      <div className="mx-auto max-w-6xl px-6 py-20">
        <div className="flex items-center justify-between mb-8">
          <h2 className="text-2xl font-semibold text-foreground">Featured Packages</h2>
          <Link href="/packages" className="text-primary hover:underline text-sm font-medium">
            View all
          </Link>
        </div>
        
        <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
          {featured.map((pkg) => (
            <Link
              key={pkg.name}
              href={`/packages/${pkg.name}`}
              className="group block p-6 rounded-lg border border-border bg-card hover:border-primary/30 transition-all h-full"
            >
              <div className="flex items-center justify-between mb-3">
                <span className="font-mono font-medium text-lg text-foreground group-hover:text-primary transition-colors">
                  {pkg.name}
                </span>
                <span className="text-xs font-mono text-muted-foreground bg-accent px-2 py-1 rounded">
                  v{pkg.version}
                </span>
              </div>
              <p className="text-muted-foreground text-sm leading-relaxed line-clamp-2">
                {pkg.description}
              </p>
              <div className="mt-4 flex items-center gap-4 text-xs text-muted-foreground/60">
                <div className="flex items-center gap-1">
                  <span className="bg-primary/10 text-primary w-2 h-2 rounded-full" />
                  {pkg.author}
                </div>
                {pkg.updated_at && (
                  <span>Updated {new Date(pkg.updated_at * 1000).toLocaleDateString()}</span>
                )}
              </div>
            </Link>
          ))}
        </div>
      </div>
    </section>
  );
}
