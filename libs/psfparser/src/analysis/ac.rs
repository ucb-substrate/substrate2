use num::complex::Complex64;
use std::collections::HashMap;

use crate::ascii::ast::{PsfAst, Trace, Values};
use crate::bin_search_before;

pub struct AcData {
    pub signals: HashMap<String, Vec<Complex64>>,
    pub freq: Vec<f64>,
}

impl AcData {
    pub fn from_ascii(ast: &PsfAst) -> Self {
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

        let mut signals = HashMap::<String, Vec<Complex64>>::new();
        let mut freq = Vec::<f64>::new();
        for v in ast.values.iter() {
            if v.signal == "freq" {
                if let Values::Real(values) = &v.values {
                    debug_assert_eq!(values.len(), 1);
                    freq.push(values[0]);
                } else {
                    panic!("Expected real signal values; found complex");
                }
            } else if let Values::Complex(values) = &v.values {
                debug_assert_eq!(values.len(), 1);
                if let Some(lst) = signals.get_mut(groups[v.signal]) {
                    lst.push(values[0]);
                } else {
                    signals.insert(groups[v.signal].to_string(), vec![values[0]]);
                }
            } else {
                panic!("Expected complex signal values; found real");
            }
        }

        Self { signals, freq }
    }

    pub fn from_binary(mut ast: crate::binary::ast::PsfAst) -> Self {
        let mut signals = HashMap::<String, Vec<Complex64>>::new();
        for group in ast.traces.iter() {
            for sig in group.signals() {
                let v = ast
                    .values
                    .as_mut()
                    .unwrap_sweep()
                    .values
                    .remove(&sig.id)
                    .expect("missing values for trace");
                signals.insert(sig.name.to_string(), v.unwrap_complex());
            }
        }

        assert_eq!(
            ast.sweeps[0].name, "freq",
            "ac analysis expects to sweep frequency"
        );
        let freq = ast
            .values
            .as_mut()
            .unwrap_sweep()
            .values
            .remove(&ast.sweeps[0].id)
            .unwrap()
            .unwrap_real();

        Self { signals, freq }
    }

    /// Gets the index into the data arrays
    /// corresponding to the largest frequency less than or equal to `f`.
    pub fn idx_before_freq(&self, f: f64) -> Option<usize> {
        bin_search_before(&self.freq, f)
    }

    #[inline]
    pub fn signal(&self, name: &str) -> Option<&Vec<Complex64>> {
        self.signals.get(name)
    }
}
