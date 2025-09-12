import z from "zod";
import { McpServer as UpstreamMCPServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { CallToolResult } from "@modelcontextprotocol/sdk/types.js";
import { httpClient } from "../../../../http_client.js";
export function setupTool<S extends UpstreamMCPServer>(server: S) {
  server.tool(
    "post_v3_images_fill_async",
    "Fills an area of an image based on images that Firefly generates based on your prompt. We call this part of an image an image, and this mask defines the area which should be filled. For example you can have an circular mask defined on an image and Firefly can generate the image of a planet in that area.\n\nPerforms this asynchronously, meaning you make your request and provide an image with your text prompt and on success, returns a <code>jobID</code>, status endpoint and endpoint to cancel the request. You can later poll the status endpoint to get updates on whether the job completes and to get the image.",
    {
      body: z.object({
        image: z.object({
          source: z.object({
            url: z.string(),
          }),
        }),
        mask: z.object({
          invert: z.boolean(),
          source: z.object({
            url: z.string(),
          }),
        }),
        negativePrompt: z.string(),
        numVariations: z.number(),
        prompt: z.string(),
        promptBiasingLocaleCode: z.string(),
        seeds: z.array(z.number()),
        size: z.object({
          height: z.number(),
          width: z.number(),
        }),
      }),
    },
    async (args: any): Promise<CallToolResult> => {
      try {
        const response = await httpClient.call({
          path: `/alerts/active/zone/{zoneId}`,
          pathParams: {
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
        console.error(`Error executing post_v3_images_fill_async:`, error);
        return {
          content: [
            {
              type: "text",
              text: `Error executing post_v3_images_fill_async: ${error instanceof Error ? error.message : String(error)}`,
            },
          ],
        };
      }
    },
  );
}
