I'm pretty sure something went wrong during latest upgrade in `soroban-env`. There's been an avalanche of questions about `Error()`

## Summary of the problem

Every time someone calls a command from other contract, it returns `Error(Value, InvalidInput)`.
I tracked it down to the example of using `transfer()` call on token contract.
While executing each other methods, error propagation works well.
But when error is being propagated from other contract's wasm binary, the callstack is becoming basically useless for debugging.


## How to reproduce

To reproduce the issue, go into `error_propagation_example` directory and call `make test`.


```
$ make test

...

running 3 tests
test test::test_from_contract - should panic ... ok
test test::transfer_insufficient_balance - should panic ... ok
test test::transfer_insufficient_balance_same_test_as_in_token_contract - should panic ... FAILED

failures:

---- test::transfer_insufficient_balance_same_test_as_in_token_contract stdout ----
thread 'test::transfer_insufficient_balance_same_test_as_in_token_contract' panicked at 'HostError: Error(Value, InvalidInput)

Event log (newest first):
   0: [Diagnostic Event] topics:[error, Error(Value, InvalidInput)], data:"escalating error to panic"
   1: [Diagnostic Event] topics:[error, Error(Value, InvalidInput)], data:["contract call failed", initialize, [Address(Contract(9540d237bdccd21be156788494c80fb741b29b1bd357a3288c223b83deb13533)), 7, "name", "symbol"]]
   2: [Failed Diagnostic Event (not emitted)] contract:aaa200c56a9b6cec122368f0fedacf8c466114c187d9511a4dfc846048bd14c1, topics:[error, Error(Value, InvalidInput)], data:["symbol not found in slice of strs", initialize]
   3: [Diagnostic Event] topics:[fn_call, Bytes(aaa200c56a9b6cec122368f0fedacf8c466114c187d9511a4dfc846048bd14c1), initialize], data:[Address(Contract(9540d237bdccd21be156788494c80fb741b29b1bd357a3288c223b83deb13533)), 7, "name", "symbol"]
   4: [Diagnostic Event] contract:aaa200c56a9b6cec122368f0fedacf8c466114c187d9511a4dfc846048bd14c1, topics:[fn_return, set_admin], data:Void
   5: [Contract Event] contract:aaa200c56a9b6cec122368f0fedacf8c466114c187d9511a4dfc846048bd14c1, topics:[set_admin, Address(Account(3427fc7a1428b047c29932eacd41e44ed6f2b33cd2c4fe90c853f0e83f8aa2ac)), "aaaa:GA2CP7D2CQULAR6CTEZOVTKB4RHNN4VTHTJMJ7UQZBJ7B2B7RKRKYKF4"], data:Address(Contract(9540d237bdccd21be156788494c80fb741b29b1bd357a3288c223b83deb13533))
   6: [Diagnostic Event] topics:[fn_call, Bytes(aaa200c56a9b6cec122368f0fedacf8c466114c187d9511a4dfc846048bd14c1), set_admin], data:Address(Contract(9540d237bdccd21be156788494c80fb741b29b1bd357a3288c223b83deb13533))
   7: [Diagnostic Event] contract:aaa200c56a9b6cec122368f0fedacf8c466114c187d9511a4dfc846048bd14c1, topics:[fn_return, init_asset], data:Void
   8: [Diagnostic Event] topics:[fn_call, Bytes(aaa200c56a9b6cec122368f0fedacf8c466114c187d9511a4dfc846048bd14c1), init_asset], data:Bytes(0000000161616161000000003427fc7a1428b047c29932eacd41e44ed6f2b33cd2c4fe90c853f0e83f8aa2ac)

Backtrace (newest first):
   0: <soroban_env_host::host::Host as soroban_env_common::env::EnvBase>::escalate_error_to_panic
             at /home/xxx/.cargo/registry/src/index.crates.io-6f17d22bba15001f/soroban-env-host-0.0.17/src/host.rs:844:26
   1: soroban_sdk::env::internal::reject_err::{{closure}}
             at /home/xxx/.cargo/registry/src/index.crates.io-6f17d22bba15001f/soroban-sdk-0.9.2/src/env.rs:52:23
   2: core::result::Result<T,E>::map_err
             at /rustc/8ede3aae28fe6e4d52b38157d7bfe0d3bceef225/library/core/src/result.rs:828:27
   3: soroban_sdk::env::internal::reject_err
             at /home/xxx/.cargo/registry/src/index.crates.io-6f17d22bba15001f/soroban-sdk-0.9.2/src/env.rs:52:9
   4: <soroban_sdk::env::Env as soroban_env_common::env::Env>::call
             at /home/xxx/.cargo/registry/src/index.crates.io-6f17d22bba15001f/soroban-sdk-0.9.2/src/env.rs:1372:13
   5: soroban_sdk::env::Env::invoke_contract
             at /home/xxx/.cargo/registry/src/index.crates.io-6f17d22bba15001f/soroban-sdk-0.9.2/src/env.rs:393:18
   6: soroban_error_propagation_example::token::Client::initialize
             at src/token.rs:4:1
   7: soroban_error_propagation_example::test::create_token
             at src/test.rs:53:5
   8: soroban_error_propagation_example::test::transfer_insufficient_balance_same_test_as_in_token_contract
             at src/test.rs:70:17
   9: soroban_error_propagation_example::test::transfer_insufficient_balance_same_test_as_in_token_contract::{{closure}}
             at src/test.rs:63:67
  10: core::ops::function::FnOnce::call_once
             at /rustc/8ede3aae28fe6e4d52b38157d7bfe0d3bceef225/library/core/src/ops/function.rs:250:5

', /home/xxx/.cargo/registry/src/index.crates.io-6f17d22bba15001f/soroban-env-host-0.0.17/src/host.rs:845:9
note: panic did not contain expected string
      panic message: `"HostError: Error(Value, InvalidInput)\n\nEvent log (newest first)...,
 expected substring: `"insufficient balance"`

failures:
    test::transfer_insufficient_balance_same_test_as_in_token_contract
```

Please check out tests and comments to see the variants I checked:
- [at first I thought that error is propagated incorrectly when called from within the other contract](https://github.com/ueco-jb/soroban-error-propagation-example/blob/master/error_propagation_example/src/test.rs#L12-L32)
- [then I checked that even exactly same testcase, but still using wasm binary, will fail with the same error](https://github.com/ueco-jb/soroban-error-propagation-example/blob/master/error_propagation_example/src/test.rs#L34-L49)
- [and at last I literally copied existing testcase from token contract, which works fine out there](https://github.com/ueco-jb/soroban-error-propagation-example/blob/master/error_propagation_example/src/test.rs#L57-L76)

Could it be that errors from other binaries are being catched under one umbrella [in soroban-env-common/src/errors.rs:161](https://github.com/stellar/rs-soroban-env/blob/844b6abc461594027158a7bb5643447d0f90a81e/soroban-env-common/src/error.rs#L161C11-L161C11)?

## Live net

```bash
$ soroban contract deploy \
--wasm target/wasm32-unknown-unknown/release/soroban_error_propagation_example.wasm \
--source alice \
--rpc-url https://rpc-futurenet.stellar.org:443 \
--network-passphrase 'Test SDF Future Network ; October 2022'
CA4R57WRHK3OMRUJOKUJEW6XQNEUR5FZOK2ZXGIILIIWRWR5S3DPFA6Q

$ soroban contract invoke \
--id CB64D3G7SM2RTH6JSGG34DDTFTQ5CFDKVDZJZSODMCX4NJ2HV2KN7OHT \ # XLM Test Token
--source alice \
--network futurenet \
-- \
balance --id $(soroban config identity address alice)
"99999789207"

$ soroban contract invoke \
--id CA4R57WRHK3OMRUJOKUJEW6XQNEUR5FZOK2ZXGIILIIWRWR5S3DPFA6Q \
--source alice \
--rpc-url https://rpc-futurenet.stellar.org:443 \
--network-passphrase 'Test SDF Future Network ; October 2022' \
-- transfers --from $(soroban config identity address alice) --to $(soroban config identity address bob) --token CB64D3G7SM2RTH6JSGG34DDTFTQ5CFDKVDZJZSODMCX4NJ2HV2KN7OHT --amount 999997892071000
error: transaction simulation failed: HostError: Error(Value, InvalidInput)
DebugInfo not available
```

## Extra issue

Tokens can be minted/transfered etc. without being initialized. I'm not sure if it's a bug or a feature.
