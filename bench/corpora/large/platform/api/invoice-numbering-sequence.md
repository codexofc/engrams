---
name: invoice-numbering-sequence
description: Invoice numbers are gapless per legal entity and year (HF-FR-2026-000123), allocated from the invoice_counters table with a row lock inside the same transaction that persists the invoice
type: reference
status: active
verified: 2026-01-09
---

# Invoice numbering

Legal requirement in France and Germany: invoice numbers must be sequential without gaps per issuing entity, per fiscal year. This is not a PostgreSQL sequence problem, because sequences have gaps on rollback.

## Format

`<ENTITY>-<YEAR>-<6 digits>`, for example `HF-FR-2026-000123` or `HF-DE-2026-000045`. Entities: `HF-FR` (Halden Freight SAS), `HF-DE` (Halden Freight GmbH), `HF-NL` (since 2026-01). The entity is chosen from `shipper.billing_entity`, never from the carrier.

Credit notes use `<ENTITY>-CN-<YEAR>-<6 digits>` and their own counter.

## Allocation

Table:

```sql
CREATE TABLE invoice_counters (
  entity text NOT NULL,
  kind text NOT NULL,   -- 'invoice' | 'credit_note'
  year int NOT NULL,
  last_value int NOT NULL DEFAULT 0,
  PRIMARY KEY (entity, kind, year)
);
```

`App\Invoicing\InvoiceNumberAllocator::next(Entity $e, Kind $k, int $year)` does, inside the caller's transaction:

```sql
INSERT INTO invoice_counters (entity, kind, year, last_value) VALUES (:e, :k, :y, 1)
ON CONFLICT (entity, kind, year) DO UPDATE SET last_value = invoice_counters.last_value + 1
RETURNING last_value;
```

The `ON CONFLICT DO UPDATE` takes a row lock, so concurrent invoice creation for the same entity serializes on that row. Throughput ceiling is roughly 200 invoices per second per entity, measured with `pgbench` in December 2025. We issue about 9 000 invoices per month. Not a concern.

The important part: the allocation and the `INSERT INTO invoices` are in the same transaction. If the invoice insert fails, the counter update rolls back too. There is a `FinalizeInvoiceHandler` that wraps both, and an `InvoiceNumberingGaplessTest` that inserts 500 invoices with 20 % forced failures and checks the numbers are contiguous.

`pg_advisory_xact_lock` was the first idea. Dropped because the counter row lock is simpler and survives PgBouncer transaction mode, see [[postgres-connection-pool-pgbouncer]].

## Drafts

Draft invoices have no number, `invoices.number IS NULL`, and `invoices.status = 'DRAFT'`. The number is assigned at finalization only. A partial unique index `uniq_invoices_number ON invoices(number) WHERE number IS NOT NULL` guards against duplicates.

## Year boundary

The year is the finalization date in the entity's timezone (`Europe/Paris` for FR and NL, `Europe/Berlin` for DE, which is the same offset but we keep them separate on purpose). Not UTC. An invoice finalized at 23:30 UTC on 31 December is a 1 January invoice in Paris. This was wrong in the first version and produced two `2025-000001` for HF-FR. The fix was HF-1188 and a manual renumbering approved by accounting.

## Never

Never `UPDATE invoices SET number = ...` by hand. If accounting needs a correction, issue a credit note and a new invoice. This is a legal constraint, not a technical preference.
