# cronox

## Setup

```bash
cargo add cronox
```

## Usage

```rs
let mut scheduler = Scheduler::new();

scheduler.call(async || {}).every_second();
scheduler.call(async || {}).every_five_seconds();
scheduler.call(async || {}).every_minute();

scheduler.await;
```
