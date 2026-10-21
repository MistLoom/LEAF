import { serveAbility } from "../../ability-sdk-ts/lib/ability.js";

await serveAbility({
  ping: async (ctx) => {
    await ctx.emit("pong", { from: "probe" });
    return { ok: true, from: "external-ts" };
  },
});
