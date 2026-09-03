use std::collections::VecDeque;

use tdi_ai::ReferenceArm;
use tdi_ai::task_execution::{SymbolicTaskAdapter, execute_symbolic_task};
use tdi_ai::task_generators::{T1Config, TaskEvent, TaskSymbol, generate_t1};

#[derive(Debug)]
struct PublicAdapter {
    predictions: VecDeque<TaskSymbol>,
}

impl SymbolicTaskAdapter for PublicAdapter {
    type Error = core::convert::Infallible;

    fn arm(&self) -> ReferenceArm {
        ReferenceArm::A1
    }

    fn reset(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn associate(&mut self, _key_code: u64, _value: TaskSymbol) -> Result<(), Self::Error> {
        Ok(())
    }

    fn payload(&mut self, _value: TaskSymbol) -> Result<(), Self::Error> {
        Ok(())
    }

    fn distractor(&mut self, _token: TaskSymbol) -> Result<(), Self::Error> {
        Ok(())
    }

    fn query_association(&mut self, _key_code: u64) -> Result<TaskSymbol, Self::Error> {
        Ok(self.predictions.pop_front().expect("scripted prediction"))
    }

    fn query_payload(&mut self, _position: u64) -> Result<TaskSymbol, Self::Error> {
        Ok(self.predictions.pop_front().expect("scripted prediction"))
    }
}

#[test]
fn public_symbolic_executor_preserves_generator_targets() {
    let instance = generate_t1(11, T1Config::new(4, 2, 2).expect("config")).expect("instance");
    let predictions = instance
        .events()
        .iter()
        .filter_map(|event| match event {
            TaskEvent::QueryAssociation { target, .. } => Some(*target),
            _ => None,
        })
        .collect();
    let mut adapter = PublicAdapter { predictions };

    let record = execute_symbolic_task(&instance, &mut adapter).expect("execution");
    assert_eq!(record.arm(), ReferenceArm::A1);
    assert_eq!(record.queries().len(), 2);
    assert!(record.all_queries_exact());
}
