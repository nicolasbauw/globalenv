# globalenv

[![Current Crates.io Version](https://img.shields.io/crates/v/globalenv.svg)](https://crates.io/crates/globalenv)
[![Downloads badge](https://img.shields.io/crates/d/globalenv.svg)](https://crates.io/crates/globalenv)

Globally sets environment variables (and not just for the current process).
Example:
```rust
use globalenv::set_var;
set_var("ENVTEST", "TESTVALUE").unwrap();
```

License: GPL-3.0
