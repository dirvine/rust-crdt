# crdts
##### thoroughly tested hybrid CRDT's.
[![Build Status](https://travis-ci.org/rust-crdt/rust-crdt.svg?branch=master)](https://travis-ci.org/rust-crdt/rust-crdt)
[![crates.io](http://meritbadge.herokuapp.com/crdts)](https://crates.io/crates/crdts)
[![docs.rs](https://docs.rs/crdts/badge.svg)](https://docs.rs/crdts)

A family of CRDT's supporting both State and Op based replication. 

# How to use this library
## Interacting with the CRDT's
Working with CRDT's is a bit different than datastructures you're used to, especially `causal` CRDT's. With `causal` CRDT's we need to make sure that your edits only effect the data that you've seen, for example, if you clear a `Map`, we want to be able to say that this clear operation will only effect entries in the map that you are aware of. If you are not tracking this causal history you could end up deleting data that you are not aware of. A potential sequence of steps that lead to lost data:
1. you receive a `Map` CRDT from across the network.
2. you read the `Map`'s key/value pairs and display them to the user.
3. you receive an update version of the `Map` CRDT but the user has not refreshed their view.
4. The user chooses to clear the values of the `Map`. So you call `Map::clear` on your CRDT.

At this point you've potentially cleared data that the user didn't want to clear. To fix this, we need to include a `causal` context with the clear operation. This causal context is a vector clock (VClock) that stores the version of the `Map` that was seen by this user when they decided to `Map::clear()`.

Lets take a look at what interacting with CRDT's looks like in using `crdts`.

First create an instance of the CRDT, we'll use the MVReg (Multi-Value Register) CRDT for this example. It allows us to store a value, when it receives conflicting `set` operations, it'll store all conflicting values.
``` rust
let mut reg = MVReg::new();
```
To set a value in your CRDT, you'll need to provide a context (even for the initial value), the only way to get a context is to first read from the CRDT.
``` rust
let read_ctx = reg.read();
assert_eq!(read_ctx.val, vec![]);
```
Reading any state from a CRDT will produces a `ReadCtx`.to access the value from the `ReadCtx`, use the `.val` field. From the example above we see the register is currently not storing any values (empty `Vec`).

Now to make your edit to the `reg`, you'll derive the appropriate context for the edit you want to make, for edits that remove data, you'll need to use `.derive_rm_ctx()`, for adding new data you'll need `.derive_add_ctx(<actor_id>)` where `<actor_id>` is a unique identifier of whatever is acting on the CRDT.

``` rust
let add_ctx = read_ctx.derive_add_ctx(123);
let rm_ctx = read_ctx.derive_rm_ctx();

reg.set("Value".to_string(), add_ctx);
reg.clear(rm_ctx);
assert_eq!(reg.read().val, vec!["Value".to_string()])
```

Now you may be wondering why we have a `"Value"` after we've cleared the register. The `"Value"` string was added with a context that included a new edit from actor `123`. The clear operation used an `RmCtx` that was derived from a read where we did not have this edit from `123`, only data that was seen at the time of the `read` is removed.

## Further reading
If you want to learn about how CRDTs work, I suggest starting with the readme from [aphyr's meangirls](https://github.com/aphyr/meangirls) repo.
Afterwards, either check out the [riak dt](https://github.com/basho/riak_dt) source code or [A comprehensive study of CRDTs](https://hal.inria.fr/file/index/docid/555588/filename/techreport.pdf) depending on if you like to read papers or jump straight to source code examples.

### references

- [A comprehensive study of CRDTs](https://hal.inria.fr/file/index/docid/555588/filename/techreport.pdf)

- [riak dt - Convergent replicated datatypes in Erlang](https://github.com/basho/riak_dt)
