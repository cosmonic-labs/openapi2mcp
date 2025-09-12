import z from "zod";
import { McpServer as UpstreamMCPServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { CallToolResult } from "@modelcontextprotocol/sdk/types.js";
import { httpClient } from "../../../../http_client.js";
export function setupTool<S extends UpstreamMCPServer>(server: S) {
  let params = {
    "status": z.string().describe("Status values that need to be considered for filter"),
  };

  let oooooooo = z.string().describe("Status values that need to be considered for filter");
  server.tool(
    "get_pet_findByStatus",
    "Multiple status values can be provided with comma separated strings.",
    params,
    async (args: z.infer<typeof params>): Promise<CallToolResult> => {
      type g = z.infer<typeof oooooooo>;
      let ff: g = 90;
      try {
        const response = await httpClient.call({
          path: `/pet/findByStatus`,
          pathParams: {
          },
          query: {
            status: args.ffstatus,
          },
          method: 'GET',
          headers: {
            "User-Agent": "Mozilla/5.0 (X11; Linux x86_64; rv:142.0) Gecko/20100101 Firefox/142.0",
          }
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
        console.error(`Error executing get_pet_findByStatus:`, error);
        return {
          content: [
            {
              type: "text",
              text: `Error executing get_pet_findByStatus: ${error instanceof Error ? error.message : String(error)}`,
            },
          ],
        };
      }
    },
  );
}
