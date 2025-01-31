use std::collections::HashMap;

use crate::ascii::ast::PsfAst as AsciiAst;
use crate::binary::ast::{PsfAst as BinaryAst, SignalValues};

#[enumify::enumify]
#[derive(Debug, Clone)]
pub enum DcData {
    Op(OpData),
    Sweep(SweepData),
}

#[derive(Debug, Clone)]
pub struct OpData {
    pub signals: HashMap<String, f64>,
}

#[derive(Debug, Clone)]
pub struct SweepData {
    /// A map from signal name to values.
    ///
    /// The ith element of each value `Vec` corresponds to a simulation
    /// with `sweep_var` set to `sweep_values[i]`.
    pub signals: HashMap<String, Vec<f64>>,
    /// The name of the variable being swept.
    pub sweep_var: String,
    /// The values of the sweep variable.
    pub sweep_values: Vec<f64>,
}

impl DcData {
    pub fn from_ascii(ast: &AsciiAst) -> Self {
        use crate::ascii::ast::{Trace, Values};
        // Assume all groups have count = 1
        // group name -> signal name
        let mut groups = HashMap::<&str, &str>::new();
        let mut i = 0;
        while i < ast.traces.len() {
            match ast.traces[i] {
                Trace::Group { name: group, count } => {
                    assert!(
                        count >= 0,
                        "trace group must contain a non-negative number of elements"
                    );
                    let count = count as usize;
                    for j in 1..=count {
                        if let Trace::Signal { name, .. } = ast.traces[i + j] {
                            groups.insert(group, name);
                        } else {
                            panic!("expected signal; found group");
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

        let sweepvar = if ast.sweeps.is_empty() {
            None
        } else {
            assert_eq!(
                ast.sweeps.len(),
                1,
                "psfparser only supports sweeping one variable"
            );
            Some(ast.sweeps[0].name.to_string())
        };
        let mut signals = HashMap::<String, Vec<f64>>::new();
        let mut sweep_values = Vec::<f64>::new();
        for v in ast.values.iter() {
            if let Values::Real(values) = &v.values {
                assert_eq!(
                    values.len(),
                    1,
                    "expected a single floating point signal value"
                );
                if Some(v.signal) == sweepvar.as_deref() {
                    sweep_values.push(values[0]);
                } else {
                    let group = groups.get(v.signal).unwrap_or(&v.signal);
                    if let Some(lst) = signals.get_mut(*group) {
                        lst.push(values[0]);
                    } else {
                        signals.insert(group.to_string(), vec![values[0]]);
                    }
                }
            } else {
                panic!("Expected real signal values; found complex");
            }
        }

        match sweepvar {
            Some(sweep_var) => Self::Sweep(SweepData {
                signals,
                sweep_var,
                sweep_values,
            }),
            None => Self::Op(OpData {
                signals: HashMap::from_iter(signals.into_iter().map(|(k, v)| (k, v[0]))),
            }),
        }
    }

    pub fn from_binary(ast: BinaryAst) -> Self {
        match ast.values {
            SignalValues::Point(values) => {
                let mut signals = HashMap::new();
                for (name, value) in values.values {
                    signals.insert(name, value.value.unwrap_real());
                }

                Self::Op(OpData { signals })
            }
            SignalValues::Sweep(mut values) => {
                assert_eq!(ast.sweeps.len(), 1);
                let swp = &ast.sweeps[0];
                let sweep_values = values.values.remove(&swp.id).unwrap().unwrap_real();
                let sweep_var = swp.name.to_string();
                let mut signals = HashMap::new();
                for trace in ast.traces.iter() {
                    for sig in trace.signals() {
                        let data = values.values.remove(&sig.id).unwrap().unwrap_real();
                        signals.insert(sig.name.to_string(), data);
                    }
                }
                Self::Sweep(SweepData {
                    signals,
                    sweep_var,
                    sweep_values,
                })
            }
        }
    }
}

impl OpData {
    #[inline]
    pub fn signal(&self, name: &str) -> Option<f64> {
        self.signals.get(name).cloned()
    }
}

impl SweepData {
    #[inline]
    pub fn signal(&self, name: &str) -> Option<&Vec<f64>> {
        self.signals.get(name)
    }
}
