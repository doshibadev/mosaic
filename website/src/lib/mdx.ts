import fs from "fs";
import path from "path";
import { compileMDX } from "next-mdx-remote/rsc";

const CONTENT_PATH = path.join(process.cwd(), "src/content/docs");

export async function getDocBySlug(slug: string[]) {
  const filePath = path.join(CONTENT_PATH, `${slug.join("/")}.mdx`);
  
  if (!fs.existsSync(filePath)) {
    return null;
  }

  const source = fs.readFileSync(filePath, "utf8");

  const { content, frontmatter } = await compileMDX<{
    title: string;
    description: string;
  }>({
    source,
    options: { parseFrontmatter: true },
  });

  return {
    content,
    frontmatter,
  };
}

export async function getAllDocSlugs() {
  const files = fs.readdirSync(CONTENT_PATH);
  return files
    .filter((file) => file.endsWith(".mdx"))
    .map((file) => file.replace(".mdx", "").split("/"));
}
