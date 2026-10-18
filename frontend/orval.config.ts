import { defineConfig } from 'orval';

export default defineConfig({
  documentService: {
    output: {
      client: "axios",
      mode: "tags-split",
      httpClient: "axios",
      target: "src/lib/api/clients.ts",
      baseUrl: "http://localhost:8123"
    },
    input: {
      "target": "./openapi.json"
    }
  }
})