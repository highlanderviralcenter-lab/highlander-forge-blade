# ADR-005: Next.js + Axum for SaaS Portal

## Status

Accepted (Future: v4.0.0)

## Context

The v4.0.0 SaaS portal requires:

1. Web dashboard for fleet management
2. REST API for report uploads and queries
3. Real-time updates (WebSocket)
4. Authentication (JWT + API keys)
5. Integration with existing Rust domain logic

## Decision

Use Next.js 14 (App Router) for the frontend and Axum (Rust) for the API backend. Share types via OpenAPI/JSON Schema.

## Consequences

### Positive

- Next.js: mature ecosystem, SSR, React Server Components
- Axum: reuse Rust types, performance, type safety
- Utoipa: auto-generated OpenAPI spec from Rust code
- Separation allows independent scaling and deployment
- Team can leverage existing Rust expertise

### Negative

- Two languages in codebase (Rust + TypeScript)
- More complex deployment (frontend + backend)
- Learning curve for frontend team if Rust-focused

## Alternatives Considered

| Alternative | Pros | Cons | Decision |
|-------------|------|------|----------|
| **Leptos (Rust → WASM)** | Single language | Immature ecosystem; limited UI components | ❌ Rejected for v4.0 |
| **Tauri + Next.js** | Desktop + web | Overkill; adds complexity | ❌ Rejected |
| **Pure Axum + HTML templates** | Simple | Limited interactivity; no modern UI | ❌ Rejected |
| **Next.js + Axum** | Best of both worlds | Two languages | ✅ Accepted |
| **Leptos (revisit v4.1+)** | Single language, Rust-native | Wait for ecosystem maturity | ⚠️ Future evaluation |

## Architecture

```
Frontend (Next.js 14)
  ├── App Router
  ├── Tailwind CSS + shadcn/ui
  ├── TanStack Query
  └── Recharts

Backend (Axum)
  ├── Utoipa (OpenAPI)
  ├── JWT + API Keys
  ├── PostgreSQL
  └── Redis
```

## API-First Design

```rust
#[derive(OpenApi)]
#[openapi(paths(get_machines), components(schemas(Machine)))]
struct ApiDoc;

#[utoipa::path(get, path = "/api/v1/machines")]
async fn get_machines() -> Json<Vec<Machine>> {
    // ...
}
```

Auto-generated TypeScript types from OpenAPI spec:

```bash
# Generate types for frontend
npx openapi-typescript https://api.hfb.dev/openapi.json --output src/api/types.ts
```

---

*Date: 2026-06-18 | Author: SaaS Team*
