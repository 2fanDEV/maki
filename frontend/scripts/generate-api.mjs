import { mkdir, writeFile } from 'node:fs/promises';
import openapiTS, { astToString } from 'openapi-typescript';

const specUrl = process.env.OPENAPI_URL ?? 'http://127.0.0.1:3000/openapi.json';
const specFile = new URL('../openapi.json', import.meta.url);
const typesFile = new URL('../src/api/schema.d.ts', import.meta.url);

let stage = 'fetch the OpenAPI specification';

try {
  const response = await fetch(specUrl, { signal: AbortSignal.timeout(10_000) });
  if (!response.ok) {
    throw new Error(`HTTP ${response.status} ${response.statusText}`);
  }

  stage = 'read the OpenAPI JSON';
  const spec = await response.json();
  stage = 'generate TypeScript types';
  const types = astToString(await openapiTS(spec));
  const json = `${JSON.stringify(spec, null, 2)}\n`;

  // Fetch and generate both outputs before replacing either existing file.
  stage = 'save the generated files';
  await mkdir(new URL('.', typesFile), { recursive: true });
  await writeFile(specFile, json);
  await writeFile(typesFile, types);
  console.log('Generated openapi.json and src/api/schema.d.ts.');
} catch (error) {
  console.error(`Could not ${stage}: ${error.message}`);
  if (stage === 'fetch the OpenAPI specification') {
    console.error('Start the Maki server or set OPENAPI_URL to its OpenAPI endpoint.');
  }
  process.exitCode = 1;
}
