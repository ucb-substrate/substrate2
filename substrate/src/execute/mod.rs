use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ToolOpts {
    /// Prelude to place at the top of tool invocation scripts.
    ///
    /// Should include a shebang, sourcing of any shell scripts,
    /// and/or environment variables.
    pub prelude: String,
    /// Flags to pass to tool invocations.
    pub flags: String,
    /// Execution control options.
    pub exec_control: ExecControlOpts,
}

#[derive(Debug, Clone)]
pub struct ExecControlOpts {
    /// Number of CPUs the tool should use.
    pub cpus: usize,
    /// Number of machines the tool should use.
    pub machines: usize,
    /// Location where tool should place output.
    pub logs: LogSource,
    /// Options to pass directly to the executor plugin.
    pub opts: HashMap<String, String>,
    /// Working directory.
    pub work_dir: PathBuf,
}

/// Place where a tool produces logs
#[derive(Default, Debug)]
pub enum LogSource {
    /// Standard output / standard error.
    #[default]
    Stdout,
    /// Logs are saved to a file at the given path.
    File(PathBuf),
}

/// Parsed Substrate.toml (after merging hierarchical config).
pub struct SubstrateConfig {}

pub struct ExecutorOutput {}

pub struct SubstrateError {}

pub trait Executor {
    fn execute(
        &self,
        command: String,
        config: &SubstrateConfig,
        opts: ExecControlOpts,
    ) -> Result<ExecutorOutput, SubstrateError>;
}

mod proof_of_concept {
    use std::process::Command;

    use super::*;

    pub struct CalibrePlugin;
    pub struct SubstrateCtx;

    pub struct DrcInput {
        opts: ToolOpts,
    }

    pub struct BsubPlugin;

    impl Executor for BsubPlugin {
        fn execute(
            &self,
            command: String,
            config: &SubstrateConfig,
            opts: ExecControlOpts,
        ) -> Result<ExecutorOutput, SubstrateError> {
            let _output = Command::new("bsub")
                .args(["-q", "bora", "-n", &opts.cpus.to_string(), "--", &command])
                .output()
                .unwrap();
            // let job_id = parse(output);
            // wait until job finishes
            Ok(ExecutorOutput {})
        }
    }

    impl CalibrePlugin {
        pub fn run_drc(&self, ctx: SubstrateCtx, input: DrcInput) {
            let mut exec_opts = input.opts.exec_control.clone();

            if exec_opts.machines != 1 {
                // warning! calibre can only run on one machine
                exec_opts.machines = 1;
            }

            if exec_opts.cpus > 32 {
                // warning! calibre can only run on 32 CPUs
                exec_opts.machines = 1;
            }

            // make drc run file
            // make bashrc
            let _file = std::fs::write(
                "run_drc.sh",
                format!("{}\ncalibre -drc layout.gds", input.opts.prelude),
            );
            // chmod +x run_drc.sh
            ctx.execute("run_drc.sh", exec_opts);
            // parse outputs
        }
    }
}
