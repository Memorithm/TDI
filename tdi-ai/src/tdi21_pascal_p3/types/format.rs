use super::{Cell, SEMANTICS};

pub fn canonical_header() -> &'static str {
    "split\tn\tk\tdensity\tschedule\tquery_load\treuse\tchecksum_pascal\tchecksum_direct\tchecksum_generic\tmismatch_pascal\tmismatch_generic\tpascal_zeta_xors\tdirect_term_tests\tgeneric_term_tests\tquery_lookups\tcoefficient_bytes\tmaterialized_table_bytes\tanf_semantic_bits\tpairwise_token_comparisons\tw_pascal\tw_direct\tw_generic\tdelta_direct\tdelta_generic\tsemantics"
}

impl Cell {
    pub fn canonical_record(&self) -> String {
        let mut fields = vec![
            self.split.name().to_owned(),
            self.n.to_string(),
            self.k.to_string(),
            self.density.to_string(),
            self.schedule.name().to_owned(),
            self.query_load.to_string(),
            self.reuse.to_string(),
        ];
        fields.extend(self.checksums.map(|x| x.to_string()));
        fields.extend(self.mismatches.map(|x| x.to_string()));
        fields.extend(self.counts.map(|x| x.to_string()));
        fields.extend(self.representation.map(|x| x.to_string()));
        fields.push(self.pairwise_token_comparisons.to_string());
        fields.extend(self.work.map(|x| x.to_string()));
        fields.extend(self.deltas.map(|x| x.to_string()));
        fields.push(SEMANTICS.to_owned());
        fields.join("\t")
    }
}
