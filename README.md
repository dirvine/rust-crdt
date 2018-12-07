# crdts: A family of thoroughly tested hybrid CRDT's.
[![Build Status](https://travis-ci.org/rust-crdt/rust-crdt.svg?branch=master)](https://travis-ci.org/rust-crdt/rust-crdt)
[![crates.io](http://meritbadge.herokuapp.com/crdts)](https://crates.io/crates/crdts)[[![docs.rs](https://docs.rs/crdts/badge.svg)](https://docs.rs/crdts)

The CRDT's here all support both State and Op based replication, use them as you need them.

# How to use this library
First create an instance of the CRDT, we'll use the MVReg CRDT in this example.
``` rust
let mut reg = MVReg::new();
```

To edit your CRDT, you'll need to provide a context, the only way to get a context is to first read from the CRDT.
``` rust
let read_ctx = reg.read();
assert_eq!(read_ctx.val, vec![]);
```

Reading anything from a CRDT will produces a `ReadCtx`. e.g. `Map::len()` returns a `ReadCtx<usize>`, to access the value from the `ReadCtx`, use the `.val` field.

Now you'll derive the appropriate context for the edit you want to make, for edits that remove data, you'll need to use `.derive_rm_ctx()`, for adding new data you'll need `.derive_add_ctx(<actor_id>)` where `<actor_id>` is a unique identifier of the device/thread/server acting on the CRDT.

``` rust
let add_ctx = read_ctx.derive_add_ctx(123);
let rm_ctx = read_ctx.derive_rm_ctx();

reg.set("Value".to_string(), add_ctx);
reg.clear(rm_ctx);
assert_eq!(reg.read().val, vec!["Value".to_string()])
```

Now you may be wondering why we have a value when we just cleared the register. This is because the `MVReg::clear` only removed data for which the `rm_ctx` has seen. The `"Value"` string was added with a context that marked a new edit from actor `123` which was unseen at the time that we first read the state of the `MVReg`.



## Further reading
If you want to learn about how CRDTs work, I suggest starting with the readme from [aphyr's meangirls](https://github.com/aphyr/meangirls) repo.
Afterwards, either check out the [riak dt](https://github.com/basho/riak_dt) source code or [A comprehensive study of CRDTs](https://hal.inria.fr/file/index/docid/555588/filename/techreport.pdf) depending on if you like to read papers or jump straight to source code examples.

### references

- [A comprehensive study of CRDTs](https://hal.inria.fr/file/index/docid/555588/filename/techreport.pdf)

- [riak dt - Convergent replicated datatypes in Erlang](https://github.com/basho/riak_dt)
