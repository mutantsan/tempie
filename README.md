# Tempie

A command-line tool for tracking time in Jira using Tempo.

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

