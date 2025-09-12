import z from "zod";
import { McpServer as UpstreamMCPServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { CallToolResult } from "@modelcontextprotocol/sdk/types.js";
import { httpClient } from "../../../../http_client.js";
export function setupTool<S extends UpstreamMCPServer>(server: S) {
  const params = {
    "status": z.string().describe("Status values that need to be considered for filter"),
  };
  type ParamsType = {
    [K in keyof typeof params]: z.infer<typeof params[K]>
  };
  server.tool(
    "get_pet_find_by_status",
    "Multiple status values can be provided with comma separated strings.",
    params,
    async (args: ParamsType): Promise<CallToolResult> => {
      try {
        const response = await httpClient.call({
          path: `/pet/findByStatus`,
          method: 'GET',
          query: {
            "status": args.status,
          },
        })
        .then((response: Response) => response.text());

        return {
          content: [
            {
              type: "text",
              text: response,
            },
          ],
        };
      } catch (error) {
        console.error(`Error executing get_pet_find_by_status:`, error);
        return {
          content: [
            {
              type: "text",
              text: `Error executing get_pet_find_by_status: ${error instanceof Error ? error.message : String(error)}`,
            },
          ],
        };
      }
    },
  );
}
