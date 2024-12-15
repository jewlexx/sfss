# sfsu telemetry

We collect totally optional telemetry regarding errors, with minimal system information to help diagnose issues.

This is to help us understand and fix bugs quicker.

## What we collect

- OS
- Architecture
- Computer Hostname
- Version
- Error message
- Logs

## How we use it

We use this data to help us prioritize features, and fix bugs quicker.

## How we protect your data

All data is stored by [Sentry](https://sentry.io/) in the EU in compliance with [GDPR](https://gdpr.eu/what-is-gdpr/).

## How we collect it

Telemetry is collected using [Sentry](https://sentry.io/).

## Opt-out

You can opt-out of telemetry by setting the `SFSU_TELEMETRY_DISABLED` environment variable to `1`, by passing the `--no-telemetry` flag, or running `sfsu telemetry off`.

Thank you for helping us make sfsu better!

## Discuss

We would love to hear your thoughts on telemetry.

You can post them [on the discussion page](https://github.com/orgs/winpax/discussions/917).
