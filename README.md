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

By default, it shows today's worklogs. You can specify a date range:

```bash
tempie list 2024-03-01 2024-03-31
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

