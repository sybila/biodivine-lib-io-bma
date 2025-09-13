[![Crates.io](https://img.shields.io/crates/v/biodivine-lib-io-bma?style=flat-square)](https://crates.io/crates/biodivine-lib-io-bma)
[![Api Docs](https://img.shields.io/badge/docs-api-yellowgreen?style=flat-square)](https://docs.rs/biodivine-lib-io-bma/)
[![Continuous integration](https://img.shields.io/github/actions/workflow/status/sybila/biodivine-lib-io-bma/build.yml?branch=main&style=flat-square)](https://github.com/sybila/biodivine-lib-io-bma/actions?query=workflow%3Abuild)
[![Coverage](https://img.shields.io/codecov/c/github/sybila/biodivine-lib-io-bma?style=flat-square)](https://codecov.io/gh/sybila/biodivine-lib-io-bma)
[![GitHub issues](https://img.shields.io/github/issues/sybila/biodivine-lib-io-bma?style=flat-square)](https://github.com/sybila/biodivine-lib-io-bma/issues)
[![GitHub last commit](https://img.shields.io/github/last-commit/sybila/biodivine-lib-io-bma?style=flat-square)](https://github.com/sybila/biodivine-lib-io-bma/commits/main)
[![Crates.io](https://img.shields.io/crates/l/biodivine-lib-io-bma?style=flat-square)](https://github.com/sybila/biodivine-lib-io-bma/blob/main/LICENSE)

# Biodivine BMA IO Library

Rust library for working with models in the [BMA format](https://biomodelanalyzer.org/).

Currently supported features:
 - Input and output from both `.json` and `.xml` BMA files (to the best of our ability, parts of the format seem
   to have changed over the years).
 - Detection of model integrity issues:
   - Invalid IDs, variable types, variable ranges, function expressions, etc.;
   - Invalid regulations and errors in regulation monotonicity;
   - Errors in function evaluation (division by zero, etc.).
 - Function evaluation, including the normalization process used by BMA.
 - Conversions between `BmaModel` and `biodivine-lib-param-bn::BooleanNetwork` (**including 
   binarization of multivalued models**).