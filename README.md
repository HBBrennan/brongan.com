## www.brongan.com

My personal website monorepo.

#### Server Rendered Routes

```
www.brongan.com/catscii
www.brongan.com/analytics
```

#### Client Rendered Routes

```
www.brongan.com/ishihara
www.brongan.com/game-of-life
www.brongan.com/mandelbrot
```

#### Dev Stack

- R u s t
- yew
- tokio
- axum
- planetscale
- fly.io
- nix/crane
- podman
- sentry
- honeycomb

#### Project Layout

- client/ contains the frontend.
- server/ contains the backend.
- shared/ contains files shared between them.

#### Development Setup

1. **Prerequisites**

   - Rust toolchain
   - [Just](https://github.com/casey/just) command runner
   - [Trunk](https://trunkrs.dev/) and [wasm-pack](https://rustwasm.github.io/wasm-pack/) (will be automatically installed if missing)

2. **Available Commands**

   ```bash
   # List all available commands
   just

   # Start both frontend and backend servers with hot reload
   just develop

   # Build all packages
   just build

   # Format code and run linters
   just format

   # Run server locally with cargo
   just local-run

   # Build and run container
   just container-run

   # Run all checks before committing
   just precommit

   # Deploy to fly.io
   just deploy
   ```

3. **Development Environment Variables**
   Create a `.env` file in the root directory with:

   ```
   ANALYTICS_DB=path/to/your/local/analytics.db
   ```

Note: The project uses workspace-level optimization settings for release builds, including link-time optimization and size optimization.
