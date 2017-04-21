extern crate cortex_m_rtfm as rtfm;

use rtfm::{C2, C4, P1, P3, Resource};

static R1: Resource<i32, C2> = Resource::new(0);

fn j1(prio: P1) {
    R1.lock(&prio, |r1, _| {
        // Would preempt this critical section
        // rtfm::request(j2);
    });
}

fn j2(prio: P3) {
    rtfm::critical(|ceil| {
        let r1 = R1.borrow(&prio, &ceil);
        //~^ error
    });
}
