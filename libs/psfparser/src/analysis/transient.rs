use std::collections::HashMap;

use crate::ascii::ast::{PsfAst as AsciiAst, Trace, Values};
use crate::bin_search_before;
use crate::binary::ast::PsfAst as BinaryAst;
use float_eq::float_eq;

#[derive(Debug, Clone, PartialEq)]
pub struct TransientData {
    pub signals: HashMap<String, Vec<f64>>,
    pub time: String,
}

impl TransientData {
    pub fn approx_eq(&self, other: &Self, reltol: f64) -> bool {
        for (name, sig) in self.signals.iter() {
            let osig = match other.signal(name) {
                Some(s) => s,
                _ => {
                    return false;
                }
            };

            if sig.len() != osig.len() {
                return false;
            }

            for (x1, x2) in sig.iter().zip(osig.iter()) {
                if !float_eq!(x1, x2, r2nd <= reltol) {
                    return false;
                }
            }
        }

        for name in other.signals.keys() {
            if self.signal(name).is_none() {
                return false;
            }
        }

        true
    }

    pub fn from_binary(mut ast: BinaryAst) -> Self {
        let mut signals = HashMap::new();
        for trace in ast.traces.iter() {
            for sig in trace.group().signals.iter() {
                let data = ast
                    .values
                    .as_mut()
                    .unwrap_sweep()
                    .values
                    .remove(&sig.id)
                    .unwrap()
                    .unwrap_real();
                signals.insert(sig.name.to_string(), data);
            }
        }

        for swp in ast.sweeps.iter() {
            let data = ast
                .values
                .as_mut()
                .unwrap_sweep()
                .values
                .remove(&swp.id)
                .unwrap()
                .unwrap_real();
            signals.insert(swp.name.to_string(), data);
        }

        Self {
            signals,
            time: "time".to_string(),
        }
    }

    pub fn from_ascii(ast: &AsciiAst) -> Self {
        // Assume all groups have count = 1
        // group name -> signal name
        let mut groups = HashMap::<&str, &str>::new();
        let mut i = 0;
        while i < ast.traces.len() {
            match ast.traces[i] {
                Trace::Group { name: group, count } => {
                    debug_assert!(count >= 0);
                    let count = count as usize;
                    for j in 1..=count {
                        if let Trace::Signal { name, .. } = ast.traces[i + j] {
                            groups.insert(group, name);
                        } else {
                            panic!("Expected signal; found group");
                        }
                    }
                    i += count + 1;
                }
                Trace::Signal { name, .. } => {
                    groups.insert(name, name);
                    i += 1;
                }
            }
        }

        groups.insert("time", "time");
        let mut signals = HashMap::<String, Vec<f64>>::new();
        for v in ast.values.iter() {
            if let Values::Real(values) = &v.values {
                debug_assert_eq!(values.len(), 1);
                if let Some(lst) = signals.get_mut(groups[v.signal]) {
                    lst.push(values[0]);
                } else {
                    signals.insert(groups[v.signal].to_string(), vec![values[0]]);
                }
            } else {
                panic!("Expected real signal values; found complex");
            }
        }

        Self {
            signals,
            time: "time".to_string(),
        }
    }

    /// Gets the index into the data arrays
    /// corresponding to the latest time less than or equal to `t`.
    pub fn idx_before_time(&self, t: f64) -> Option<usize> {
        bin_search_before(self.signal(&self.time).unwrap(), t)
    }

    #[inline]
    pub fn signal(&self, name: &str) -> Option<&Vec<f64>> {
        self.signals.get(name)
    }
}
