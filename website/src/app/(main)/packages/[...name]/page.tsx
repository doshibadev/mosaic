import { getPackage } from "@/lib/registry";
import { notFound } from "next/navigation";
import Link from "next/link";
import { ArrowLeft, Github, Package } from "lucide-react";
import ReactMarkdown from "react-markdown";

interface PackagePageProps {
  params: Promise<{
    name: string[];
  }>;
}

export default async function PackagePage({ params }: PackagePageProps) {
  // Next.js 15+ made params async. The name comes in as an array because of the
  // catch-all route [name].tsx. Join it to handle scoped packages like "scope/name".
  const { name } = await params;
  const packageName = name.join("/");
  const pkg = await getPackage(packageName);

  if (!pkg) {
    notFound();
  }

  return (
    <div className="min-h-screen">
      <div className="mx-auto max-w-7xl px-6 py-12">
        {/* Back link */}
        <Link
          href="/packages"
          className="inline-flex items-center gap-2 text-muted-foreground/60 hover:text-foreground transition-colors mb-8 text-sm font-medium"
        >
          <ArrowLeft className="h-4 w-4" />
          Back to packages
        </Link>

        {/* Header section with package name and latest version */}
        <div className="flex flex-col md:flex-row md:items-start md:justify-between gap-6 mb-10">
          <div>
            <div className="flex items-center gap-3 mb-3">
              <Package className="h-6 w-6 text-primary" />
              <h1 className="text-3xl font-bold text-foreground font-mono">{pkg.name}</h1>
            </div>
            <p className="text-lg text-muted-foreground leading-relaxed max-w-2xl">
              {pkg.description}
            </p>
          </div>
          <div className="shrink-0 bg-card border border-border rounded-lg px-5 py-4 min-w-[200px]">
            <div className="text-sm text-muted-foreground/60 mb-1">Latest version</div>
            <div className="text-2xl font-bold font-mono text-foreground">{pkg.version}</div>
          </div>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-3 gap-10">
          {/* Left column: documentation/readme */}
          <div className="lg:col-span-2 space-y-10">
            {/* Documentation (README) */}
            {pkg.readme ? (
              <div>
                <h2 className="text-lg font-semibold text-foreground mb-4">Documentation</h2>
                {/* ReactMarkdown with custom prose styling for dark mode */}
                <article className="prose prose-invert max-w-none prose-headings:font-bold prose-a:text-primary prose-code:text-primary prose-code:bg-muted/50 prose-code:px-1 prose-code:rounded prose-pre:bg-card prose-pre:border prose-pre:border-border">
                  <ReactMarkdown>{pkg.readme}</ReactMarkdown>
                </article>
              </div>
            ) : (
               <div>
                <h2 className="text-lg font-semibold text-foreground mb-4">Documentation</h2>
                <p className="text-muted-foreground italic">No README provided.</p>
               </div>
            )}
          </div>

          {/* Right column: installation and metadata */}
          <div className="space-y-10">
            {/* Install command */}
            <div>
              <h2 className="text-lg font-semibold text-foreground mb-3">Install</h2>
              <div className="bg-card border border-border rounded-lg p-4">
                <div className="flex items-center justify-between">
                  <div className="font-mono text-base break-all">
                    <span className="text-muted-foreground/50 select-none">$ </span>
                    <span className="text-foreground">mosaic install {pkg.name}@{pkg.version}</span>
                  </div>
                </div>
              </div>
              <p className="text-sm text-muted-foreground/60 mt-2">
                Or add to your <code className="text-primary bg-accent px-1.5 py-0.5 rounded text-sm font-mono">mosaic.toml</code> manually:
              </p>
              <div className="bg-card border border-border rounded-lg p-4 mt-2 overflow-x-auto scrollbar-hide">
                <pre className="font-mono text-sm text-foreground whitespace-pre">
                  <span className="text-muted-foreground">[dependencies]</span>{"\n"}
                  <span className="text-foreground">{pkg.name.includes("/") ? `"${pkg.name}"` : pkg.name}</span>
                  <span className="text-muted-foreground"> = </span>
                  <span className="text-primary">&quot;{pkg.version}&quot;</span>
                </pre>
              </div>
            </div>

            {/* Usage example - shows how to actually use the package in Polytoria */}
            <div>
              <h2 className="text-lg font-semibold text-foreground mb-3">Usage</h2>
              <div className="bg-card border border-border rounded-lg p-4 overflow-x-auto scrollbar-hide">
                <pre className="font-mono text-sm text-foreground leading-relaxed whitespace-pre">
                  <span className="text-muted-foreground">-- In your Polytoria script</span>{"\n"}
                  <span className="text-primary">local</span>
                  <span className="text-foreground"> {toPascalCase(pkg.name)} = </span>
                  <span className="text-secondary">require</span>
                  <span className="text-foreground">(game[</span>
                  <span className="text-primary">&quot;ScriptService&quot;</span>
                  <span className="text-foreground">]</span>
                  {getErrorPathArgs(pkg.name)}
                  <span className="text-foreground">)</span>
                </pre>
              </div>
            </div>

            {/* Metadata grid: author, license, repo, downloads */}
            <div className="grid grid-cols-1 gap-4">
              <div className="bg-card border border-border rounded-lg p-5">
                <h3 className="text-sm text-muted-foreground/60 mb-2">Author</h3>
                <p className="text-base text-foreground font-medium">{pkg.author}</p>
              </div>
              <div className="bg-card border border-border rounded-lg p-5">
                <h3 className="text-sm text-muted-foreground/60 mb-2">License</h3>
                <p className="text-base text-foreground font-medium">MIT</p>
              </div>
              {pkg.repository && (
                <div className="bg-card border border-border rounded-lg p-5">
                  <h3 className="text-sm text-muted-foreground/60 mb-2">Repository</h3>
                  <Link
                    href={pkg.repository}
                    target="_blank"
                    className="inline-flex items-center gap-2 text-secondary hover:underline text-base"
                  >
                    <Github className="h-4 w-4" />
                    {pkg.repository}
                  </Link>
                </div>
              )}
              <div className="bg-card border border-border rounded-lg p-5">
                <h3 className="text-sm text-muted-foreground/60 mb-2">Downloads</h3>
                <p className="text-base text-foreground font-medium">{pkg.download_count.toLocaleString()}</p>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

/// Converts a package name to PascalCase for use in Lua.
/// Handles scoped packages: "scope/package" -> "Package"
/// Example: "logger-util" -> "LoggerUtil", "my/logger" -> "Logger"
function toPascalCase(str: string): string {
  const name = str.includes("/") ? str.split("/")[1] : str;
  return name
    .split(/[-_]/)
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join("");
}

/// Generates the correct require path for scoped vs non-scoped packages.
/// Scoped packages like "scope/logger" use nested table access: ["scope"]["logger"]
/// Regular packages like "logger" use single access: ["logger"]
function getErrorPathArgs(name: string) {
  if (name.includes("/")) {
    const [scope, pkg] = name.split("/");
    return (
      <>
        <span className="text-foreground">[</span>
        <span className="text-primary">&quot;{scope}&quot;</span>
        <span className="text-foreground">][</span>
        <span className="text-primary">&quot;{pkg}&quot;</span>
        <span className="text-foreground">]</span>
      </>
    );
  }
  return (
    <>
      <span className="text-foreground">[</span>
      <span className="text-primary">&quot;{name}&quot;</span>
      <span className="text-foreground">]</span>
    </>
  );
}