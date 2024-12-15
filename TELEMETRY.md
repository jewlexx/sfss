# sfsu telemetry

We collect totally optional and completely anonymous telemetry regarding errors.

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

We don't collect any personally identifiable information.

We don't collect any information about your usage of the tool.

We don't collect any information about your errors.

All data is stored by [Sentry](https://sentry.io/) in the [https://gdpr.eu/what-is-gdpr/].

## How we collect it

Telemetry is collected using [Sentry](https://sentry.io/).

## Opt-out

You can opt-out of telemetry by setting the `SFSU_TELEMETRY_DISABLED` environment variable to `1`, by passing the `--no-telemetry` flag, or running `sfsu telemetry off`.

Thank you for helping us make sfsu better!