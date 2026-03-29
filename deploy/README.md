# Deployment Notes

## Reverse Proxy and TLS
- Use `deploy/nginx.conf` with real certificates in `deploy/certs`.
- Terminate TLS at Nginx and forward to the app on port 8080.

## Database Migrations
- In production set `APP_ENV=production` and `RUN_MIGRATIONS=false`.
- Run migrations as a separate deployment step before starting the app.

## Monitoring
- Use `/api/health` for health checks.
- Collect stdout/stderr logs from the container or systemd service.
- Add external uptime checks and alerting for `/api/health` and process restarts.

## Backups
- Schedule `deploy/backup.ps1` on Windows Task Scheduler or replace with pg_dump in your Linux scheduler.
- Keep encrypted off-host backups.

## Secrets
- Do not commit production `.env` files.
- Store secrets in your host secret manager or deployment platform.

## Reconciliation Worker
- The app now starts a background reconciliation worker that re-checks pending payments with stored tx hashes.
- Tune `RECONCILIATION_INTERVAL_SECONDS` based on traffic and chain confirmation expectations.

## Integration Tests
- Set `TEST_DATABASE_URL` and run `cargo test -- --ignored` to execute the real-Postgres integration scaffold.
