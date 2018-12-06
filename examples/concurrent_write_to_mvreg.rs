extern crate crdts;

use crdts::{CvRDT, CmRDT, MVReg};

fn main() {
    let mut reg: MVReg<String, u8> = Map::new();
    let op = reg.set(
        "Initial value of the register",
        reg.read().derive_add_ctx(1)
    );
    reg.apply(&op);

    // copy the reg over to another device
    let reg2 = reg.clone();
    let add_ctx2 = reg2.read().derive_add_ctx(2);
    let op2 = reg_on_device2.set("Update reg on device2", add_ctx2);
    reg_on_device2.apply(&op2);

    {
        let add_ctx = map.len()
            .derive_add_ctx(1);

        let op = map.update(
            "bob",
            add_ctx,
            |set, c| set.add("is feeling O.K.", c)
        );
        map.apply(&op);
    }

    let mut map_on_device2 = map.clone();
    // the map on the 2nd devices adds to the set
    // under the "bob" key
    {
        let device2_add_ctx = map_on_device2
            .len()
            .derive_add_ctx(2);
        let op = map_on_device2.update(
            "bob",
            device2_add_ctx,
            |set, c| set.add("is overwhelmed", c)
        );
        map_on_device2.apply(&op);
    }
    // concurrently the map on the first device
    // remove 'bob'
    {
        let rm_ctx = map
            .get(&"bob".to_string())
            .derive_rm_ctx();
        map.rm("bob", rm_ctx);
    }

    // once these two devices synchronize...
    map.merge(&map_on_device2);
    map_on_device2.merge(&map);

    // we see that "bob" is present but the
    // set under bob only contains the changes
    // unseen by the first map

    let val = map
        .get(&"bob".to_string()).val
        .map(|set| set.read().val);
    assert_eq!(
        val,
        Some(
            // only one entry left
            vec!["is overwhelmed".to_string()]
                .into_iter()
                .collect()
        )
    );
}
