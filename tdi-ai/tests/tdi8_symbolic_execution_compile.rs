use std::collections::VecDeque;

use tdi_ai::ReferenceArm;
use tdi_ai::task_execution::{SymbolicTaskAdapter, TaskPrediction, execute_symbolic_task};
use tdi_ai::task_generators::{T1Config, TaskEvent, TaskSymbol, generate_t1};

#[derive(Debug)]
struct PublicAdapter {
    predictions: VecDeque<TaskPrediction>,
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

    fn query_association(&mut self, _key_code: u64) -> Result<TaskPrediction, Self::Error> {
        Ok(self.predictions.pop_front().expect("scripted prediction"))
    }

    fn query_payload(&mut self, _position: u64) -> Result<TaskPrediction, Self::Error> {
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
            TaskEvent::QueryAssociation { target, .. } => Some(TaskPrediction::Symbol(*target)),
            _ => None,
        })
        .collect();
    let mut adapter = PublicAdapter { predictions };

    let record = execute_symbolic_task(&instance, &mut adapter).expect("execution");
    assert_eq!(record.arm(), ReferenceArm::A1);
    assert_eq!(record.queries().len(), 2);
    assert_eq!(record.invalid_predictions(), 0);
    assert!(record.all_queries_exact());
}

#[test]
fn public_symbolic_executor_counts_invalid_prediction_as_failure() {
    let instance = generate_t1(13, T1Config::new(3, 2, 1).expect("config")).expect("instance");
    let mut adapter = PublicAdapter {
        predictions: VecDeque::from([TaskPrediction::Invalid]),
    };

    let record = execute_symbolic_task(&instance, &mut adapter).expect("execution");
    assert_eq!(record.queries().len(), 1);
    assert_eq!(record.invalid_predictions(), 1);
    assert_eq!(record.failed_queries(), 1);
    assert!(!record.all_queries_exact());
}
