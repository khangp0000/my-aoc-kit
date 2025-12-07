# aoc-cli

Command-line interface for running Advent of Code solvers.

> *"Because copy-pasting from a browser is for people who value their time."*

## Features

- Run solvers by year/day/part with flexible filtering
- Automatic input fetching from adventofcode.com
- Per-user input caching (never fetch the same input twice)
- Answer submission with throttle handling
- Parallel execution at year/day/part granularity
- Tag-based solver filtering
- Ordered result output (results always print in year/day/part order)

## Installation

```bash
cargo install --path aoc-cli
```

Or run directly from the workspace:

```bash
cargo run -p aoc-cli -- [OPTIONS]
```

## Usage

```bash
# Run all solvers
aoc

# Run specific year
aoc --year 2024

# Run specific day
aoc --year 2024 --day 1

# Run specific part
aoc --year 2024 --day 1 --part 1

# Run with tag filter
aoc --tags easy,parsing

# Submit answers
aoc --year 2024 --day 1 --submit

# Quiet mode (answers only)
aoc --year 2024 --day 1 --quiet
```

## Options

| Option | Short | Description |
|--------|-------|-------------|
| `--year <YEAR>` | `-y` | Year to run (all years if omitted) |
| `--day <DAY>` | `-d` | Day to run, 1-25 (all days if omitted) |
| `--part <PART>` | `-p` | Part to run (all parts if omitted) |
| `--tags <TAGS>` | `-t` | Comma-separated tags to filter solvers |
| `--cache-dir <PATH>` | | Cache directory (default: `~/.cache/aoc_solver`) |
| `--threads <N>` | | Number of threads (default: CPU count) |
| `--parallelize-by <LEVEL>` | | Parallelization: `sequential`, `year`, `day`, `part` (default: `day`) |
| `--submit` | | Submit answers to adventofcode.com |
| `--user-id <ID>` | | User ID for cache organization |
| `--auto-retry` | | Auto-retry on throttle with parsed wait time |
| `--quiet` | `-q` | Quiet mode - only output answers |

## Session Token

The CLI needs your AOC session token to fetch inputs and submit answers. Set it via environment variable:

```bash
export AOC_SESSION="your_session_cookie_here"
```

To get your session cookie:
1. Log in to [adventofcode.com](https://adventofcode.com)
2. Open browser dev tools (F12) → Application → Cookies
3. Copy the `session` cookie value

If no session is set, the CLI will prompt for it when needed.

## Input Caching

Inputs are cached per-user at `~/.cache/aoc_solver/{user_id}/{year}_day{day}.txt`.

The user ID is automatically fetched from your session, or you can provide it with `--user-id`.

## Parallelization

Control how solvers run in parallel:

- `sequential` - No parallelization, run everything in order
- `year` - Parallelize across years; days/parts sequential within each year
- `day` (default) - Parallelize across year/day; parts sequential
- `part` - Maximum parallelization across all year/day/part combinations

```bash
# Run everything sequentially
aoc --parallelize-by sequential

# Maximum parallelization
aoc --parallelize-by part --threads 8
```

## License

MIT
