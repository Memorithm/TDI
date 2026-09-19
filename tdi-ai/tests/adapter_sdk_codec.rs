use tdi_ai::adapter_sdk::{
    AdapterContract, CodecConformanceError, ReplayCodec, Reproduction, check_codec_conformance,
};
use tdi_ai::experiment::{ReplayAdapter, StepContext};

#[derive(Clone, Debug, PartialEq, Eq)]
struct StatefulCheckpoint {
    value: u64,
    rng_state: u64,
    cached_mix: u64,
    depth: usize,
}

struct StatefulCodec {
    checkpoint: StatefulCheckpoint,
    complete_codec: bool,
}

impl StatefulCodec {
    fn new(complete_codec: bool) -> Self {
        Self {
            checkpoint: StatefulCheckpoint {
                value: 7,
                rng_state: 0,
                cached_mix: 0,
                depth: 0,
            },
            complete_codec,
        }
    }
}
impl ReplayAdapter for StatefulCodec {
    type Checkpoint = StatefulCheckpoint;
    type Observation = (u64, u64);
    type Error = &'static str;

    fn fork(&self, checkpoint: &Self::Checkpoint) -> Result<Self, Self::Error> {
        Ok(Self {
            checkpoint: checkpoint.clone(),
            complete_codec: self.complete_codec,
        })
    }

    fn checkpoint(&self) -> Result<Self::Checkpoint, Self::Error> {
        Ok(self.checkpoint.clone())
    }

    fn advance(&mut self, context: StepContext) -> Result<Self::Observation, Self::Error> {
        if context.depth != self.checkpoint.depth + 1 {
            return Err("non-sequential depth");
        }
        self.checkpoint.rng_state = self
            .checkpoint
            .rng_state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(context.noise_stream + 1);
        self.checkpoint.cached_mix = self
            .checkpoint
            .cached_mix
            .rotate_left(9)
            .wrapping_add(self.checkpoint.rng_state ^ self.checkpoint.value);
        self.checkpoint.value = self
            .checkpoint
            .value
            .wrapping_add(self.checkpoint.cached_mix);
        self.checkpoint.depth = context.depth;
        Ok((self.checkpoint.value, self.checkpoint.cached_mix))
    }
}

impl ReplayCodec for StatefulCodec {
    fn contract(&self) -> AdapterContract {
        AdapterContract {
            implementation: "tdi-sdk-stateful-codec-test/v1",
            advancement_unit: "fixture step",
            observation_unit: "fixture word",
            precision: "exact u64",
            reproduction: Reproduction::ExactBuildTarget,
            rng: "test LCG; complete state must be checkpointed",
            max_checkpoint_bytes: 40,
            max_steps: 16,
            max_state_scalars: 4,
            access: "test fixture only",
            execution_limits: "bounded in-process conformance fixture",
        }
    }

    fn progress(&self) -> usize {
        self.checkpoint.depth
    }

    fn encode_checkpoint(&self) -> Result<Vec<u8>, Self::Error> {
        let mut bytes = b"TDISDK1\0".to_vec();
        bytes.extend(self.checkpoint.value.to_le_bytes());
        if self.complete_codec {
            bytes.extend(self.checkpoint.rng_state.to_le_bytes());
            bytes.extend(self.checkpoint.cached_mix.to_le_bytes());
        }
        bytes.extend((self.checkpoint.depth as u64).to_le_bytes());
        Ok(bytes)
    }

    fn decode_checkpoint(&self, bytes: &[u8]) -> Result<Self::Checkpoint, Self::Error> {
        let expected_len = if self.complete_codec { 40 } else { 24 };
        if bytes.len() != expected_len || &bytes[..8] != b"TDISDK1\0" {
            return Err("invalid checkpoint");
        }
        let word = |offset: usize| {
            u64::from_le_bytes(
                bytes[offset..offset + 8]
                    .try_into()
                    .expect("bounded fixture"),
            )
        };
        let (rng_state, cached_mix, depth_offset) = if self.complete_codec {
            (word(16), word(24), 32)
        } else {
            (0, 0, 16)
        };
        let depth = usize::try_from(word(depth_offset)).map_err(|_| "depth overflow")?;
        if depth > 16 {
            return Err("depth bound");
        }
        Ok(StatefulCheckpoint {
            value: word(8),
            rng_state,
            cached_mix,
            depth,
        })
    }
}

fn contexts() -> Vec<StepContext> {
    (1..=4)
        .map(|depth| StepContext {
            depth,
            noise_stream: (depth % 2) as u64,
        })
        .collect()
}

#[test]
fn stepwise_codec_conformance_accepts_complete_rng_and_cache_state() {
    assert_eq!(
        check_codec_conformance(&StatefulCodec::new(true), &contexts()),
        Ok(())
    );
}
#[test]
fn stepwise_codec_conformance_rejects_codec_that_drops_runtime_state() {
    // The incomplete codec round-trips the fresh source because both mutable
    // fields start at zero. It becomes invalid only after the first advance.
    assert_eq!(
        check_codec_conformance(&StatefulCodec::new(false), &contexts()),
        Err(CodecConformanceError::CodecMismatch)
    );
}
