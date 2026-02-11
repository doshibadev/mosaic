import type { NextConfig } from "next";
import nextra from "nextra";

const withNextra = nextra({
  contentDirBasePath: "/docs",
});

const nextConfig: NextConfig = {
  /* config options here */
  reactCompiler: true,
};

export default withNextra(nextConfig);
