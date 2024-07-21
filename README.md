# Token 2022 wrapper

This is a program to wrap Token2022 Solana tokens into SPL tokens. Every token created using the Token2022 can be wrapped into its unique SPL token which can be used across several applications on Solana. This is an early release and does not include support for Extensions.

### Building and Testing Locally

To build the contract, and generate IDL + SDK, run:

```
./build.sh
```

To run the tests, run:

```
./test.sh
```