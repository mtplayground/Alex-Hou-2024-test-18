# Alex-Hou-2024-test-18

## SQLx CLI

Install the Postgres-only SQLx CLI:

```bash
cargo install sqlx-cli --no-default-features --features postgres
```

Run backend migrations against the configured database:

```bash
export DATABASE_URL=$(cat /workspace/.database_url)
sqlx migrate run --source backend/migrations
```
