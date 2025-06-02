// Vercel Edge API handler that loads and calls our Rust WebAssembly module
import wasm from './{{project_name}}_bg.wasm';
import { handler } from '../pkg/{{project_name}}.js';

// Initialize the WebAssembly module
let wasmModule;

// Our edge function handler
export default async function(req) {
  // Initialize the WebAssembly module if it's not already loaded
  if (!wasmModule) {
    // Initialize the WASM module with the imported binary
    wasmModule = await WebAssembly.instantiate(wasm);
  }
  
  // Call our Rust handler with the request
  return handler(req);
}

// Configure the runtime to use the Edge Runtime
export const config = {
  runtime: 'edge',
};
