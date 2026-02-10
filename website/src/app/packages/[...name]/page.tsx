import { getPackage } from "@/lib/registry";
import { notFound } from "next/navigation";
import Link from "next/link";
import { ArrowLeft, Github, Package } from "lucide-react";

interface PackagePageProps {
  params: Promise<{
    name: string[];
  }>;
}

export default async function PackagePage({ params }: PackagePageProps) {
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

        {/* Header */}
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

        {/* Install command */}
        <div className="mb-10">
          <h2 className="text-lg font-semibold text-foreground mb-3">Install</h2>
          <div className="bg-card border border-border rounded-lg p-4 max-w-lg">
            <div className="flex items-center justify-between">
              <div className="font-mono text-base">
                <span className="text-muted-foreground/50 select-none">$ </span>
                <span className="text-foreground">mosaic install {pkg.name}@{pkg.version}</span>
              </div>
            </div>
          </div>
          <p className="text-sm text-muted-foreground/60 mt-2">
            Or add to your <code className="text-primary bg-accent px-1.5 py-0.5 rounded text-sm font-mono">mosaic.toml</code> manually:
          </p>
          <div className="bg-card border border-border rounded-lg p-4 mt-2 max-w-lg overflow-x-auto scrollbar-hide">
            <pre className="font-mono text-sm text-foreground whitespace-pre">
              <span className="text-muted-foreground">[dependencies]</span>{"\n"}
              <span className="text-foreground">{pkg.name.includes("/") ? `"${pkg.name}"` : pkg.name}</span>
              <span className="text-muted-foreground"> = </span>
              <span className="text-primary">&quot;{pkg.version}&quot;</span>
            </pre>
          </div>
        </div>

        {/* Usage example */}
        <div className="mb-10">
          <h2 className="text-lg font-semibold text-foreground mb-3">Usage</h2>
          <div className="bg-card border border-border rounded-lg p-4 max-w-lg overflow-x-auto scrollbar-hide">
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

        {/* Details grid */}
        <div className="grid sm:grid-cols-2 gap-4 mb-10">
          <div className="bg-card border border-border rounded-lg p-5">
            <h3 className="text-sm text-muted-foreground/60 mb-2">Author</h3>
            <p className="text-base text-foreground font-medium">{pkg.author}</p>
          </div>
          <div className="bg-card border border-border rounded-lg p-5">
            <h3 className="text-sm text-muted-foreground/60 mb-2">License</h3>
            <p className="text-base text-foreground font-medium">MIT</p>
          </div>
          {pkg.repository && (
            <div className="bg-card border border-border rounded-lg p-5 sm:col-span-2">
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
          {pkg.downloads !== undefined && (
            <div className="bg-card border border-border rounded-lg p-5">
              <h3 className="text-sm text-muted-foreground/60 mb-2">Downloads</h3>
              <p className="text-base text-foreground font-medium">{pkg.downloads.toLocaleString()}</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

function toPascalCase(str: string): string {
  // Handle scoped packages: "scope/package" -> "package"
  const name = str.includes("/") ? str.split("/")[1] : str;
  return name
    .split(/[-_]/)
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join("");
}

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
