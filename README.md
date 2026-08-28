# Tempie

A command-line tool for tracking time in Jira using Tempo.

The `tempie list` command output example:

```
┌───────────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                         April 24h40m/176h (-151h20m)                                          │
├───────────────────────────────────────────────────────────────────────────────────────────────────────────────┤
│                                              Monday (2025-04-07)                                              │
├────────┬──────────┬─────────────────────┬───────────────────┬─────────────────────────────────────────────────┤
│ ID     │ Duration │ Created At          │ Description       │ Issue URL                                       │
├────────┼──────────┼─────────────────────┼───────────────────┼─────────────────────────────────────────────────┤
│ 150937 │ 10m      │ 2025-04-07 09:42:21 │ Daily meeting     │ https://xxx.jira.com/browse/ST-16               │
├────────┼──────────┼─────────────────────┼───────────────────┼─────────────────────────────────────────────────┤
│ 150938 │ 10m      │ 2025-04-07 09:47:26 │ Review solution   │ https://xxx.jira.com/browse/DCD-52              │
├────────┼──────────┼─────────────────────┼───────────────────┼─────────────────────────────────────────────────┤
│ 150941 │ 1h10m    │ 2025-04-07 11:07:25 │ All stuff meeting │ https://xxx.jira.com/browse/ST-16               │
├────────┴──────────┴─────────────────────┴───────────────────┴─────────────────────────────────────────────────┤
│                                                                                                      1h30m/8h │
└───────────────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

## Installation

```bash
cargo install tempie
```
Or install a specific version:

```bash
cargo install tempie@0.3.1
```

## Usage

### Setup

First, you need to configure your Jira credentials:

```bash
tempie setup
```

This will guide you through the setup process.

### Log Time

Log time to a Jira issue:

```bash
tempie log XXX-123 1h30m "Worked on feature implementation"
```

Arguments:
- `XXX-123`: Jira issue key
- `1h30m`: Time spent (e.g 30m, 1h30m, 1d)
- `"Worked on..."`: Optional description

### List Worklogs

View your worklogs:

```bash
tempie list
```

By default, it shows today's worklogs. You can specify a date:

```bash
tempie list 2024-03-01
```

### List Worklogs by Date Range

View worklogs for a specific date range:

```bash
tempie list-range 2024-03-01 2024-03-31
```

### Monthly Summary

Show how many hours you logged each day next to your target working hours, so
you can see what you still owe:

```bash
tempie month
```

By default it shows the current month. Pass a month (or any date within it) to
see that month instead:

```bash
tempie month 2024-03
tempie month 2024-03-01
```

Example output:

```
August 2026

      Date          Logged    Target      Diff
  Mon 2026-08-03       8h         8h        0h
  Tue 2026-08-04     7h30m        8h      -30m
  Sat 2026-08-08         ·         ·
  ...
  ──────────────────────────────────────────────
  Total            148h15m      168h   -19h45m
```

### Delete Worklog

Remove a worklog by its ID(s). You can provide one or more IDs:

```bash
tempie delete 12345 67890
```

You can get the ID from the `list` command.


## Development

### Building

```bash
cargo build
```

### Running Tests

```bash
cargo test
```

## License

MIT License - see [LICENSE](LICENSE) for details.
