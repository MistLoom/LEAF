export const inject = ["tools"];

export function apply(ctx) {
  ctx.tools.register({
    name: "cordis_echo",
    description: "Cordis 回显",
    parameters: {
      type: "object",
      properties: { msg: { type: "string" } },
      required: ["msg"],
    },
    output: {
      schema: {
        type: "object",
        properties: { msg: { type: "string" } },
        required: ["msg"],
      },
      render: (_args, value) => [{ type: "text", text: String(value?.msg ?? "") }],
    },
    async execute(args) {
      return { msg: args?.msg ?? "" };
    },
  });
}
