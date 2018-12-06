# rust-crdt
[![Build Status](https://travis-ci.org/rust-crdt/rust-crdt.svg?branch=master)](https://travis-ci.org/rust-crdt/rust-crdt)
[![crates.io](http://meritbadge.herokuapp.com/crdts)](https://crates.io/crates/crdts)

A family of thoroughly tested CRDT's.

[documentation](https://docs.rs/crdts/1.2.11/crdts/)

- **VClock**: Vector clock. Typically used to track the causal history
- **ORSWOT**: Observed-Remove Set Without Tombstones. An add-biased set
- **Map**: Add biased Map with reset-remove semantics. Map values are also CRDT's
- **MVReg**: Multi-Value Register. Holds a value, concurrent edits are delt with by storing both.
- **LWWReg**: Last-Write-Wins Register. Holds a value, concurrent edits are resolved by keeping the value with the largest counter.
- **GCounter**: Grow-only Counter. A counter that only goes up.
- **PNCounter**: Pos/Neg Counter. A counter that can go up and down.
- **GSet**: Grow-only set. A set that only grows


#### Things we still need to build (PR's welcome)
- a sequence CRDT
- ability to configure the bias of add-biased CRDT's. Can we choose a remove bias.

## examples

### OR-Set Without Tombstones (ORSWOT)
```rust
let mut a = Orswot::new();
let mut b = Orswot::new();
a.add("value bar".to_string(), "witnessing node A".to_string());
assert_eq!(a.value(), vec!["value bar".to_string()]);
b.add("value baz".to_string(), "witnessing node B".to_string());
assert_eq!(b.value(), vec!["value baz".to_string()]);
let mut c = a.clone();
assert_eq!(c.value(), vec!["value bar".to_string()]);
c.merge(b);
assert_eq!(c.value(), vec!["value bar".to_string(), "value baz".to_string()]);
unsafe { a.remove("value bar".to_string()); }
let mut d = a.clone();
d.merge(c);
assert_eq!(d.value(), vec!["value baz".to_string()]);
```


If you want to learn about how CRDTs work, I suggest starting with the readme from [aphyr's meangirls](https://github.com/aphyr/meangirls) repo.
Afterwards, either check out the [riak dt](https://github.com/basho/riak_dt) source code or [A comprehensive study of CRDTs](https://hal.inria.fr/file/index/docid/555588/filename/techreport.pdf) depending on if you like to read papers or jump straight to source code examples.


### references

- [A comprehensive study of CRDTs](https://hal.inria.fr/file/index/docid/555588/filename/techreport.pdf)

- [riak dt - Convergent replicated datatypes in Erlang](https://github.com/basho/riak_dt)
