#![cfg(feature = "experimental")]

use tdi_ai::experimental::tdi21::{BooleanState, MemoryRead};
use tdi_ai::experimental::tdi21_stream::{
    BooleanStream, Event, MemoryMode, StepOutput, StreamConfig,
};

fn machine(mode: MemoryMode) -> BooleanStream {
    BooleanStream::new(StreamConfig {
        mode,
        slots: 4,
        payload_bits: 8,
        max_events: 16,
        route_salt: 0,
    })
    .unwrap()
}

fn hit(value: u64) -> StepOutput {
    StepOutput::Reply(MemoryRead::Hit(BooleanState::from_bits(value)))
}

#[test]
fn two_way_does_not_dominate_direct_on_every_retrieval() {
    let mut direct = machine(MemoryMode::Direct);
    let mut two_way = machine(MemoryMode::TwoWay);
    for key in [1, 3, 5, 9] {
        let event = Event::Write {
            key,
            payload: BooleanState::from_bits(key),
            marker: BooleanState::from_bits(1),
        };
        direct.step(event).unwrap();
        two_way.step(event).unwrap();
    }
    // The independent task truth is that all four facts were written. B2's
    // separate direct slots preserve key 3; B3's shared bucket evicts it.
    let direct_three = direct.step(Event::Recall { key: 3 }).unwrap();
    let two_way_three = two_way.step(Event::Recall { key: 3 }).unwrap();
    println!("replacement-tradeoff: key=3;B2={direct_three:?};B3={two_way_three:?}");
    assert_eq!(direct_three, hit(3));
    assert_eq!(two_way_three, StepOutput::Reply(MemoryRead::Miss));
    // Positive cross-control: B3 preserves key 5, which B2 evicted.
    assert_eq!(
        direct.step(Event::Recall { key: 5 }).unwrap(),
        StepOutput::Reply(MemoryRead::Miss)
    );
    assert_eq!(two_way.step(Event::Recall { key: 5 }).unwrap(), hit(5));
    assert_eq!(direct.step(Event::Recall { key: 9 }).unwrap(), hit(9));
    assert_eq!(two_way.step(Event::Recall { key: 9 }).unwrap(), hit(9));
    assert_eq!(direct.counters().work.pairwise_comparisons, 0);
    assert_eq!(two_way.counters().work.pairwise_comparisons, 0);
}
