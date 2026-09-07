//! Non-final TDI-11.1 controlled-world reference evaluator.
//!
//! This layer does not choose a model, controller, threshold, verifier, seed
//! population, confidence interval, or final-evaluation surface. It receives an
//! already-generated non-final task, one externally supplied response, declared
//! action/resource accounting, and provenance identities. It then applies the
//! exact controlled-world scorer and emits a deterministic evaluator record.

use crate::hallucination_generator::{GeneratedTask, GENERATOR_SCHEMA};
use crate::hallucination_world::{HallucinationRejection, PrimaryTaskFamily, SupportLabel};

pub const EVALUATOR_SCHEMA: &str = "tdi11-reference-evaluator-v1";
pub const RESOURCE_ACCOUNTING_SCHEMA: &str = "tdi11-reference-accounting-v1";
pub const HASH_SCHEMA: &str = "tdi11-fnv1a64-v1";
const EXPECTED_ORACLE_SCHEMA: &str = "tdi11-controlled-world-v1";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ActionCounts {
    pub continue_count: u64,
    pub verify_count: u64,
    pub resample_count: u64,
    pub backtrack_count: u64,
    pub recover_count: u64,
    pub emit_count: u64,
    pub abstain_count: u64,
}

impl ActionCounts {
    #[must_use]
    pub const fn single_emit() -> Self {
        Self {
            emit_count: 1,
            ..Self::zero()
        }
    }

    #[must_use]
    pub const fn single_abstain() -> Self {
        Self {
            abstain_count: 1,
            ..Self::zero()
        }
    }

    #[must_use]
    pub const fn zero() -> Self {
        Self {
            continue_count: 0,
            verify_count: 0,
            resample_count: 0,
            backtrack_count: 0,
            recover_count: 0,
            emit_count: 0,
            abstain_count: 0,
        }
    }

    #[must_use]
    pub const fn with_continue(mut self, count: u64) -> Self {
        self.continue_count = count;
        self
    }

    #[must_use]
    pub const fn with_verify(mut self, count: u64) -> Self {
        self.verify_count = count;
        self
    }

    #[must_use]
    pub const fn with_resample(mut self, count: u64) -> Self {
        self.resample_count = count;
        self
    }

    #[must_use]
    pub const fn with_backtrack(mut self, count: u64) -> Self {
        self.backtrack_count = count;
        self
    }

    #[must_use]
    pub const fn with_recover(mut self, count: u64) -> Self {
        self.recover_count = count;
        self
    }

    fn checked_total(self) -> Option<u64> {
        [
            self.continue_count,
            self.verify_count,
            self.resample_count,
            self.backtrack_count,
            self.recover_count,
            self.emit_count,
            self.abstain_count,
        ]
        .into_iter()
        .try_fold(0_u64, u64::checked_add)
    }

    fn terminal_is_consistent(self, response: &str) -> bool {
        if response.trim() == "ABSTAIN" {
            self.abstain_count == 1 && self.emit_count == 0
        } else {
            self.emit_count == 1 && self.abstain_count == 0
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ReferenceResourceUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub decode_steps: u64,
    pub resamples: u64,
    pub verifier_calls: u64,
    pub verifier_operations: u64,
    pub retrieval_tool_calls: u64,
    pub retrieval_payload_bytes: u64,
    pub backtracks: u64,
    pub recoveries: u64,
    pub replayed_steps: u64,
    pub estimator_operations: u64,
    pub controller_state_bytes: u64,
    pub checkpoint_bytes: u64,
}

impl ReferenceResourceUsage {
    #[must_use]
    pub const fn with_tokens(mut self, input: u64, output: u64) -> Self {
        self.input_tokens = input;
        self.output_tokens = output;
        self
    }

    #[must_use]
    pub const fn with_decode_steps(mut self, count: u64) -> Self {
        self.decode_steps = count;
        self
    }

    #[must_use]
    pub const fn with_resamples(mut self, count: u64) -> Self {
        self.resamples = count;
        self
    }

    #[must_use]
    pub const fn with_verifier(mut self, calls: u64, operations: u64) -> Self {
        self.verifier_calls = calls;
        self.verifier_operations = operations;
        self
    }

    #[must_use]
    pub const fn with_retrieval(mut self, calls: u64, payload_bytes: u64) -> Self {
        self.retrieval_tool_calls = calls;
        self.retrieval_payload_bytes = payload_bytes;
        self
    }

    #[must_use]
    pub const fn with_recovery(
        mut self,
        backtracks: u64,
        recoveries: u64,
        replayed_steps: u64,
    ) -> Self {
        self.backtracks = backtracks;
        self.recoveries = recoveries;
        self.replayed_steps = replayed_steps;
        self
    }

    #[must_use]
    pub const fn with_estimator_operations(mut self, operations: u64) -> Self {
        self.estimator_operations = operations;
        self
    }

    #[must_use]
    pub const fn with_state_bytes(mut self, controller: u64, checkpoint: u64) -> Self {
        self.controller_state_bytes = controller;
        self.checkpoint_bytes = checkpoint;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReferenceResourceEnvelope {
    pub max_input_tokens: u64,
    pub max_output_tokens: u64,
    pub max_decode_steps: u64,
    pub max_resamples: u64,
    pub max_verifier_calls: u64,
    pub max_verifier_operations: u64,
    pub max_retrieval_tool_calls: u64,
    pub max_retrieval_payload_bytes: u64,
    pub max_backtracks: u64,
    pub max_recoveries: u64,
    pub max_replayed_steps: u64,
    pub max_estimator_operations: u64,
    pub max_controller_state_bytes: u64,
    pub max_checkpoint_bytes: u64,
}

impl ReferenceResourceEnvelope {
    #[must_use]
    pub const fn unlimited_for_development() -> Self {
        Self {
            max_input_tokens: u64::MAX,
            max_output_tokens: u64::MAX,
            max_decode_steps: u64::MAX,
            max_resamples: u64::MAX,
            max_verifier_calls: u64::MAX,
            max_verifier_operations: u64::MAX,
            max_retrieval_tool_calls: u64::MAX,
            max_retrieval_payload_bytes: u64::MAX,
            max_backtracks: u64::MAX,
            max_recoveries: u64::MAX,
            max_replayed_steps: u64::MAX,
            max_estimator_operations: u64::MAX,
            max_controller_state_bytes: u64::MAX,
            max_checkpoint_bytes: u64::MAX,
        }
    }

    fn contains(self, usage: ReferenceResourceUsage) -> bool {
        usage.input_tokens <= self.max_input_tokens
            && usage.output_tokens <= self.max_output_tokens
            && usage.decode_steps <= self.max_decode_steps
            && usage.resamples <= self.max_resamples
            && usage.verifier_calls <= self.max_verifier_calls
            && usage.verifier_operations <= self.max_verifier_operations
            && usage.retrieval_tool_calls <= self.max_retrieval_tool_calls
            && usage.retrieval_payload_bytes <= self.max_retrieval_payload_bytes
            && usage.backtracks <= self.max_backtracks
            && usage.recoveries <= self.max_recoveries
            && usage.replayed_steps <= self.max_replayed_steps
            && usage.estimator_operations <= self.max_estimator_operations
            && usage.controller_state_bytes <= self.max_controller_state_bytes
            && usage.checkpoint_bytes <= self.max_checkpoint_bytes
    }

    fn canonical_record(self) -> String {
        format!(
            "in={};out={};decode={};resample={};verify={};verify_ops={};tool={};payload={};backtrack={};recover={};replay={};estimator={};state={};checkpoint={}",
            self.max_input_tokens,
            self.max_output_tokens,
            self.max_decode_steps,
            self.max_resamples,
            self.max_verifier_calls,
            self.max_verifier_operations,
            self.max_retrieval_tool_calls,
            self.max_retrieval_payload_bytes,
            self.max_backtracks,
            self.max_recoveries,
            self.max_replayed_steps,
            self.max_estimator_operations,
            self.max_controller_state_bytes,
            self.max_checkpoint_bytes,
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferenceProvenanceContext {
    repository_commit: String,
    preregistration_blob: String,
    prompt_serializer_version: String,
    policy_version: String,
    policy_configuration: String,
    verifier_version: String,
    verifier_configuration: String,
}

impl ReferenceProvenanceContext {
    #[must_use]
    pub fn new(
        repository_commit: impl Into<String>,
        preregistration_blob: impl Into<String>,
        prompt_serializer_version: impl Into<String>,
    ) -> Self {
        Self {
            repository_commit: repository_commit.into(),
            preregistration_blob: preregistration_blob.into(),
            prompt_serializer_version: prompt_serializer_version.into(),
            policy_version: "none".to_owned(),
            policy_configuration: "none".to_owned(),
            verifier_version: "none".to_owned(),
            verifier_configuration: "none".to_owned(),
        }
    }

    #[must_use]
    pub fn with_policy(
        mut self,
        version: impl Into<String>,
        configuration: impl Into<String>,
    ) -> Self {
        self.policy_version = version.into();
        self.policy_configuration = configuration.into();
        self
    }

    #[must_use]
    pub fn with_verifier(
        mut self,
        version: impl Into<String>,
        configuration: impl Into<String>,
    ) -> Self {
        self.verifier_version = version.into();
        self.verifier_configuration = configuration.into();
        self
    }

    fn is_complete(&self) -> bool {
        [
            self.repository_commit.as_str(),
            self.preregistration_blob.as_str(),
            self.prompt_serializer_version.as_str(),
            self.policy_version.as_str(),
            self.policy_configuration.as_str(),
            self.verifier_version.as_str(),
            self.verifier_configuration.as_str(),
        ]
        .into_iter()
        .all(|value| !value.is_empty())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferenceEvaluationRecord {
    family: PrimaryTaskFamily,
    seed: u64,
    label: Option<SupportLabel>,
    rejection: Option<HallucinationRejection>,
    unsupported_emit: bool,
    answered: bool,
    task_success: bool,
    false_abstention: bool,
    actions: ActionCounts,
    usage: ReferenceResourceUsage,
    provenance_record: String,
    result_record_hash: String,
}

impl ReferenceEvaluationRecord {
    #[must_use]
    pub const fn family(&self) -> PrimaryTaskFamily {
        self.family
    }

    #[must_use]
    pub const fn seed(&self) -> u64 {
        self.seed
    }

    #[must_use]
    pub const fn label(&self) -> Option<SupportLabel> {
        self.label
    }

    #[must_use]
    pub fn rejection(&self) -> Option<&HallucinationRejection> {
        self.rejection.as_ref()
    }

    #[must_use]
    pub const fn unsupported_emit(&self) -> bool {
        self.unsupported_emit
    }

    #[must_use]
    pub const fn answered(&self) -> bool {
        self.answered
    }

    #[must_use]
    pub const fn task_success(&self) -> bool {
        self.task_success
    }

    #[must_use]
    pub const fn false_abstention(&self) -> bool {
        self.false_abstention
    }

    #[must_use]
    pub const fn actions(&self) -> ActionCounts {
        self.actions
    }

    #[must_use]
    pub const fn usage(&self) -> ReferenceResourceUsage {
        self.usage
    }

    #[must_use]
    pub fn provenance_record(&self) -> &str {
        &self.provenance_record
    }

    #[must_use]
    pub fn result_record_hash(&self) -> &str {
        &self.result_record_hash
    }

    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.rejection.is_none()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AggregateCounts {
    pub total_records: u64,
    pub valid_records: u64,
    pub rejected_records: u64,
    pub unsupported_emits: u64,
    pub answered: u64,
    pub unsupported_answered: u64,
    pub task_successes: u64,
    pub false_abstentions_f1_f2: u64,
}

pub fn aggregate_counts(records: &[ReferenceEvaluationRecord]) -> Option<AggregateCounts> {
    let mut output = AggregateCounts::default();
    for record in records {
        output.total_records = output.total_records.checked_add(1)?;
        if !record.is_valid() {
            output.rejected_records = output.rejected_records.checked_add(1)?;
            continue;
        }
        output.valid_records = output.valid_records.checked_add(1)?;
        if record.unsupported_emit() {
            output.unsupported_emits = output.unsupported_emits.checked_add(1)?;
        }
        if record.answered() {
            output.answered = output.answered.checked_add(1)?;
            if record.unsupported_emit() {
                output.unsupported_answered = output.unsupported_answered.checked_add(1)?;
            }
        }
        if record.task_success() {
            output.task_successes = output.task_successes.checked_add(1)?;
        }
        if record.false_abstention() {
            output.false_abstentions_f1_f2 =
                output.false_abstentions_f1_f2.checked_add(1)?;
        }
    }
    Some(output)
}

pub fn evaluate_response(
    task: &GeneratedTask,
    response: &str,
    actions: ActionCounts,
    usage: ReferenceResourceUsage,
    envelope: ReferenceResourceEnvelope,
    provenance: &ReferenceProvenanceContext,
) -> ReferenceEvaluationRecord {
    let family = task.cell().family;
    let seed = task.seed();
    let mut rejection = validate_contract(task, response, actions, usage, envelope, provenance);
    let mut label = None;
    let mut unsupported_emit = false;
    let mut answered = false;

    if rejection.is_none() {
        match task.world().score_text(response) {
            Ok(score) => {
                label = Some(score.label());
                unsupported_emit = score.unsupported_emit();
                answered = score.answered();
            }
            Err(error) => rejection = Some(error),
        }
    }

    let task_success = rejection.is_none()
        && match (family, label) {
            (
                PrimaryTaskFamily::F1ExplicitSupport
                | PrimaryTaskFamily::F2DerivedSupport
                | PrimaryTaskFamily::F4ContradictionOverride,
                Some(SupportLabel::SupportedExplicit | SupportLabel::SupportedDerived),
            ) => true,
            (
                PrimaryTaskFamily::F3HiddenTruthInsufficiency,
                Some(SupportLabel::Abstained),
            ) => true,
            _ => false,
        };

    let false_abstention = rejection.is_none()
        && matches!(
            family,
            PrimaryTaskFamily::F1ExplicitSupport | PrimaryTaskFamily::F2DerivedSupport
        )
        && label == Some(SupportLabel::Abstained);

    let provenance_record = canonical_provenance(
        task,
        provenance,
        envelope,
        actions,
        usage,
        label,
        rejection.as_ref(),
        unsupported_emit,
        answered,
        task_success,
        false_abstention,
    );
    let result_record_hash = stable_hash_hex(&provenance_record);

    ReferenceEvaluationRecord {
        family,
        seed,
        label,
        rejection,
        unsupported_emit,
        answered,
        task_success,
        false_abstention,
        actions,
        usage,
        provenance_record,
        result_record_hash,
    }
}

fn validate_contract(
    task: &GeneratedTask,
    response: &str,
    actions: ActionCounts,
    usage: ReferenceResourceUsage,
    envelope: ReferenceResourceEnvelope,
    provenance: &ReferenceProvenanceContext,
) -> Option<HallucinationRejection> {
    if !task
        .world()
        .policy_view()
        .canonical_record()
        .starts_with(EXPECTED_ORACLE_SCHEMA)
        || GENERATOR_SCHEMA != "tdi11-controlled-task-generator-v1"
    {
        return Some(HallucinationRejection::ContractDrift);
    }
    if !provenance.is_complete() {
        return Some(HallucinationRejection::ProvenanceFailure);
    }
    if actions.checked_total().is_none() {
        return Some(HallucinationRejection::ResourceAccountingOverflow);
    }
    if !actions.terminal_is_consistent(response)
        || usage.resamples != actions.resample_count
        || usage.verifier_calls != actions.verify_count
        || usage.backtracks != actions.backtrack_count
        || usage.recoveries != actions.recover_count
    {
        return Some(HallucinationRejection::ContractDrift);
    }
    if !envelope.contains(usage) {
        return Some(HallucinationRejection::ResourceEnvelopeExceeded);
    }
    None
}

#[allow(
    clippy::too_many_arguments,
    reason = "the canonical provenance record deliberately enumerates frozen evaluator fields"
)]
fn canonical_provenance(
    task: &GeneratedTask,
    provenance: &ReferenceProvenanceContext,
    envelope: ReferenceResourceEnvelope,
    actions: ActionCounts,
    usage: ReferenceResourceUsage,
    label: Option<SupportLabel>,
    rejection: Option<&HallucinationRejection>,
    unsupported_emit: bool,
    answered: bool,
    task_success: bool,
    false_abstention: bool,
) -> String {
    let visible_evidence = task
        .world()
        .policy_view()
        .evidence()
        .iter()
        .map(|entry| entry.canonical_record())
        .collect::<Vec<_>>()
        .join(";");
    let rules = task
        .world()
        .policy_view()
        .rules()
        .iter()
        .map(|rule| rule.canonical_record())
        .collect::<Vec<_>>()
        .join(";");
    let policy_hash = stable_hash_hex(&format!(
        "{}|{}",
        provenance.policy_version, provenance.policy_configuration
    ));
    let verifier_hash = stable_hash_hex(&format!(
        "{}|{}",
        provenance.verifier_version, provenance.verifier_configuration
    ));
    let label = label.map(label_name).unwrap_or("NONE");
    let rejection = rejection
        .map(ToString::to_string)
        .unwrap_or_else(|| "NONE".to_owned());

    format!(
        "{EVALUATOR_SCHEMA};series=TDI-11.1;repo_commit={};prereg_blob={};oracle_schema={EXPECTED_ORACLE_SCHEMA};generator_schema={GENERATOR_SCHEMA};prompt_serializer={};domain={};seed={:016x};family={:?};stratum={:?};hash_schema={HASH_SCHEMA};visible_hash={};rules_hash={};query_hash={};policy_version={};policy_config_hash={policy_hash};verifier_version={};verifier_config_hash={verifier_hash};accounting_schema={RESOURCE_ACCOUNTING_SCHEMA};envelope_hash={};label={label};rejection={rejection};unsupported_emit={};answered={};task_success={};false_abstention={};actions={:?};usage={:?}",
        provenance.repository_commit,
        provenance.preregistration_blob,
        provenance.prompt_serializer_version,
        task.domain().as_str(),
        task.seed(),
        task.cell().family,
        task.cell().stratum,
        stable_hash_hex(&visible_evidence),
        stable_hash_hex(&rules),
        stable_hash_hex(&task.query().canonical_record()),
        provenance.policy_version,
        provenance.verifier_version,
        stable_hash_hex(&envelope.canonical_record()),
        u8::from(unsupported_emit),
        u8::from(answered),
        u8::from(task_success),
        u8::from(false_abstention),
        actions,
        usage,
    )
}

fn label_name(label: SupportLabel) -> &'static str {
    match label {
        SupportLabel::SupportedExplicit => "SUPPORTED_EXPLICIT",
        SupportLabel::SupportedDerived => "SUPPORTED_DERIVED",
        SupportLabel::TrueHiddenUnsupported => "TRUE_HIDDEN_UNSUPPORTED",
        SupportLabel::Contradicted => "CONTRADICTED",
        SupportLabel::Nonexistent => "NONEXISTENT",
        SupportLabel::Indeterminate => "INDETERMINATE",
        SupportLabel::Abstained => "ABSTAINED",
    }
}

fn stable_hash_hex(bytes: &str) -> String {
    // Stable development-only provenance hash. This is intentionally named and
    // versioned; it is not a cryptographic authenticity primitive and may be
    // replaced by a separately frozen final-result hash scheme before any final
    // confirmation is armed.
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}")
}
