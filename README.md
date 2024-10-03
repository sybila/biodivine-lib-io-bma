# Biodivine BMA Data

Rust library for working with models in BMA format.

> Work in progress.

This library should offer functionality for:
- Parsing BMA models from JSON and XML. We aim to support the newest BMA JSON format, but also try to handle older JSON and XML variants.
- Creating new BMA models and exporting them into the latest BMA JSON format.
- Converting BMA models into `RegulatoryGraph` and `BooleanNetwork` instances of `lib-param-bn`.
  - This includes processing BMA real-number update expressions into BDD-based counterparts.
- Converting `BooleanNetwork` instances into BMA models.
- Translating between BMA and AEON model formats.