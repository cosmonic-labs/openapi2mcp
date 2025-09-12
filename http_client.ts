import { BASE_URL } from "./constants";

export interface HTTPClientParams {
    baseUrl: string;
}

export interface CallParams {
    path: string;
    pathParams?: Record<string, string>;
    query?: Record<string, string>;
    method?: RequestInit['method'];
    headers?: RequestInit['headers'];
    body?: RequestInit['body'];
}

export class HTTPClient {
    private baseUrl: string;

    constructor(params: HTTPClientParams) {
        this.baseUrl = params.baseUrl;
    }

    public call(params: CallParams): Promise<Response> {
        let path = params.path;
        for (const [key, value] of Object.entries(params.pathParams ?? {})) {
            path = path.replace(`{${key}}`, value);
        }
        console.assert(!path.includes('{'), `Not all path params were replaced in path: ${path}`);

        return fetch(`${this.baseUrl}${path}?${new URLSearchParams(params.query).toString()}`, {
            method: params.method,
            headers: params.headers,
            body: params.body,
        });
    }
}

export const httpClient = new HTTPClient({
    baseUrl: BASE_URL,
});
