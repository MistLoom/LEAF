import { existsSync } from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const hostRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const anchor = pathToFileURL(path.join(hostRoot, "package.json")).href;

/** 插件目录里的 `@deepseek-ai/*` 统一解析到宿主 node_modules，避免半套依赖。 */
export async function resolve(specifier, context, nextResolve) {
  if (specifier.startsWith("@deepseek-ai/")) {
    const pkg = specifier.split("/").slice(0, 2).join("/");
    const pkgJson = path.join(hostRoot, "node_modules", pkg, "package.json");
    if (existsSync(pkgJson)) {
      return nextResolve(specifier, { ...context, parentURL: anchor });
    }
  }
  return nextResolve(specifier, context);
}
