# YATIS - Yet Another TBank Investment Sdk

## Targets

- more usability
- easy to use any types of generated proto of  [investAPI]
- ability to create your own implementation of common traits, like `InvestService`
- easy to use pool objects

## Goals

- [x] Investing api implementation
- [x] Unary operations (all)
- [x] Pool of reusable connections
- [ ] Sandbox API
- [x] Server side streams (all)
  - [x] Authomatic reconnect on stucked connections
  - [x] Resubscribtion on reconnect
  - [ ] Connections limitation by tariff
- [ ] Bidirecional streams
  - [ ] Balanced pool of streams
  - [ ] Authomatic reconnect on stucked connections
  - [ ] Resubscribtion on reconnect

[investAPI]: https://github.com/RussianInvestments/investAPI/tree/124813610a9dbb0d8c91067a67d9c26a02c8c713/src/docs/contracts
