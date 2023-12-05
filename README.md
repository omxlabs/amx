# OMX Contracts on Arbitrum

Don't forget to pull all submodules:

```test
git submodule update --init --recursive
```

And install cargo-make:

```test
cargo install cargo-make
```

## Testing

All tests located at [scripts/tests](scripts/tests/README.md).

**Before running tests** you need to build contracts:

```test
cargo make optimize
```

**After that** you can **run** tests:

```test
cargo make test-contracts
# or this if you have mac with arm64
cargo make test-contracts-m1
```
