import { serveAbility } from "../../ability-sdk-ts/lib/ability.js";

await serveAbility({
  run: async (_ctx, _args) => ({ ok: true }),
});
