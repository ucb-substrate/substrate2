use crate::utils::{aggregate_sources, execute_run_script};
use crate::{error::Error, RuleCheck, TEMPLATES};
use regex::Regex;
use serde::Serialize;
use std::fs;
use std::io::{self, BufRead};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use tera::Context;

pub struct LvsParams<'a> {
    pub work_dir: &'a Path,
    pub layout_path: &'a Path,
    pub layout_cell_name: &'a str,
    pub source_paths: &'a [PathBuf],
    pub source_cell_name: &'a str,
    pub rules_dir: &'a Path,
    pub rules_path: &'a Path,
}

pub struct LvsGeneratedPaths {
    pub run_file_path: PathBuf,
    pub lvs_rpt_path: PathBuf,
    pub erc_summary_path: PathBuf,
}

#[derive(Serialize)]
struct LvsTemplateContext<'a> {
    layout_path: &'a Path,
    layout_cell_name: &'a str,
    source_path: &'a Path,
    source_cell_name: &'a str,
    lvs_db_path: &'a Path,
    lvs_rpt_path: &'a Path,
    erc_db_path: &'a Path,
    erc_summary_path: &'a Path,
}

pub struct LvsData {
    pub status: LvsStatus,
    pub erc_rule_checks: Vec<RuleCheck>,
}

#[derive(PartialEq, Eq, Debug, Copy, Clone, Hash)]
pub enum LvsStatus {
    Correct,
    Incorrect,
}

pub fn write_lvs_run_file(params: &LvsParams) -> Result<LvsGeneratedPaths, Error> {
    fs::create_dir_all(params.work_dir).map_err(Error::Io)?;

    let source_path = params.work_dir.join("source.spice");

    aggregate_sources(&source_path, params.source_paths)?;

    let lvs_db_path = params.work_dir.join("lvs_results.db");
    let lvs_rpt_path = params.work_dir.join("lvs_results.rpt");
    let erc_db_path = params.work_dir.join("erc_results.db");
    let erc_summary_path = params.work_dir.join("erc.summary");
    let run_file_path = params.work_dir.join("pegasuslvsctl");
    let view_lvs_path = params.work_dir.join("view_lvs.sh");
    let macro_path = params.work_dir.join("dr.mac");

    let lvs_context = LvsTemplateContext {
        layout_path: params.layout_path,
        layout_cell_name: params.layout_cell_name,
        source_path: &source_path,
        source_cell_name: params.source_cell_name,
        lvs_db_path: &lvs_db_path,
        lvs_rpt_path: &lvs_rpt_path,
        erc_db_path: &erc_db_path,
        erc_summary_path: &erc_summary_path,
    };

    let lvs_contents = TEMPLATES
        .render(
            "pegasuslvsctl",
            &Context::from_serialize(&lvs_context).map_err(Error::Tera)?,
        )
        .map_err(Error::Tera)?;

    fs::write(&run_file_path, lvs_contents).map_err(Error::Io)?;

    let mut context = Context::new();
    context.insert("layout_path", params.layout_path);
    context.insert("macro_path", &macro_path);

    let contents = TEMPLATES
        .render("view_drc_lvs.sh", &context)
        .map_err(Error::Tera)?;

    fs::write(&view_lvs_path, contents).map_err(Error::Io)?;
    let mut perms = fs::metadata(&view_lvs_path)
        .map_err(Error::Io)?
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&view_lvs_path, perms).map_err(Error::Io)?;

    let mut context = Context::new();
    context.insert("work_dir", params.work_dir);

    let contents = TEMPLATES.render("dr.mac", &context).map_err(Error::Tera)?;

    fs::write(&macro_path, contents).map_err(Error::Io)?;

    Ok(LvsGeneratedPaths {
        run_file_path,
        lvs_rpt_path,
        erc_summary_path,
    })
}

pub fn write_lvs_run_script(
    work_dir: impl AsRef<Path>,
    run_file_path: impl AsRef<Path>,
    rules_dir: impl AsRef<Path>,
    rules_path: impl AsRef<Path>,
) -> Result<PathBuf, Error> {
    fs::create_dir_all(&work_dir).map_err(Error::Io)?;

    let run_script_path = work_dir.as_ref().join("run_lvs.sh");

    let mut context = Context::new();
    context.insert("run_file_path", run_file_path.as_ref());
    context.insert("rules_dir", rules_dir.as_ref());
    context.insert("rules_path", rules_path.as_ref());

    let contents = TEMPLATES
        .render("run_lvs.sh", &context)
        .map_err(Error::Tera)?;

    fs::write(&run_script_path, contents).map_err(Error::Io)?;

    Ok(run_script_path)
}

pub fn run_pegasus_lvs(
    work_dir: impl AsRef<Path>,
    run_script_path: impl AsRef<Path>,
) -> Result<(), Error> {
    fs::create_dir_all(&work_dir).map_err(Error::Io)?;

    execute_run_script(run_script_path.as_ref(), &work_dir, "lvs")
}

pub fn parse_pegasus_lvs_results(
    lvs_rpt_path: impl AsRef<Path>,
    erc_summary_path: impl AsRef<Path>,
) -> Result<LvsData, Error> {
    let re = Regex::new(r"MISMATCH").unwrap();
    let lvs_rpt_path = lvs_rpt_path.as_ref();
    let mut ext = lvs_rpt_path.extension().unwrap_or_default().to_owned();
    ext.push(".cls");
    let lvs_rpt = fs::File::open(&lvs_rpt_path.with_extension(ext)).map_err(Error::Io)?;
    let correct = !(io::BufReader::new(lvs_rpt).lines().any(|s| {
        if let Ok(line) = s {
            re.is_match(&line)
        } else {
            false
        }
    }));

    let re = Regex::new(r"^RULECHECK (.+) \.* Total Result .* (\d+) \(.*(\d+)\)").unwrap();
    let erc_summary = fs::File::open(&erc_summary_path).map_err(Error::Io)?;
    let erc_rule_checks: Vec<RuleCheck> = io::BufReader::new(erc_summary)
        .lines()
        .filter_map(|s| {
            if let Ok(line) = s {
                re.captures(&line).map(|caps| RuleCheck {
                    name: caps.get(1).unwrap().as_str().to_string(),
                    num_results: caps.get(2).unwrap().as_str().parse().unwrap(),
                })
            } else {
                None
            }
        })
        .filter(|rule_check| rule_check.num_results > 0)
        .collect();

    Ok(LvsData {
        status: if correct {
            LvsStatus::Correct
        } else {
            LvsStatus::Incorrect
        },
        erc_rule_checks,
    })
}

pub fn run_lvs(params: &LvsParams) -> Result<LvsData, Error> {
    let LvsGeneratedPaths {
        run_file_path,
        lvs_rpt_path,
        erc_summary_path,
        ..
    } = write_lvs_run_file(params)?;
    let run_script_path = write_lvs_run_script(
        params.work_dir,
        run_file_path,
        params.rules_dir,
        params.rules_path,
    )?;
    run_pegasus_lvs(params.work_dir, run_script_path)?;
    parse_pegasus_lvs_results(lvs_rpt_path, erc_summary_path)
}

#[cfg(test)]
mod tests {
    use crate::lvs::{
        parse_pegasus_lvs_results, run_lvs, write_lvs_run_file, LvsParams, LvsStatus,
    };
    use crate::tests::{EXAMPLES_PATH, SKY130_LVS, SKY130_LVS_RULES_PATH, TEST_BUILD_PATH};
    use std::path::PathBuf;

    #[test]
    fn test_write_lvs_run_file() -> anyhow::Result<()> {
        let layout_path = PathBuf::from(EXAMPLES_PATH).join("gds/test_col_inv_array.gds");
        let source_path = PathBuf::from(EXAMPLES_PATH).join("spice/test_col_inv_array.gds");
        let work_dir = PathBuf::from(TEST_BUILD_PATH).join("test_write_lvs_run_file");

        write_lvs_run_file(&LvsParams {
            work_dir: &work_dir,
            layout_path: &layout_path,
            layout_cell_name: "test_col_inv_array",
            source_paths: &[source_path],
            source_cell_name: "col_inv_array",
            rules_dir: &PathBuf::from(SKY130_LVS),
            rules_path: &PathBuf::from(SKY130_LVS_RULES_PATH),
        })?;
        Ok(())
    }

    #[test]
    fn test_parse_pegasus_lvs_results() -> anyhow::Result<()> {
        let lvs_rpt_correct_path = PathBuf::from(EXAMPLES_PATH).join("lvs/lvs_results_correct.rpt");
        let lvs_rpt_incorrect_path =
            PathBuf::from(EXAMPLES_PATH).join("lvs/lvs_results_incorrect.rpt");
        let erc_summary_path = PathBuf::from(EXAMPLES_PATH).join("lvs/erc.summary");

        let data_correct = parse_pegasus_lvs_results(lvs_rpt_correct_path, &erc_summary_path)?;
        assert!(
            matches!(data_correct.status, LvsStatus::Correct),
            "LVS result should have been parsed as correct, but was not"
        );

        let data_incorrect = parse_pegasus_lvs_results(lvs_rpt_incorrect_path, erc_summary_path)?;
        assert!(
            matches!(data_incorrect.status, LvsStatus::Incorrect),
            "LVS result should have been parsed as incorrect, but was not"
        );
        Ok(())
    }

    #[test]
    fn test_run_lvs_col_inv() -> anyhow::Result<()> {
        let layout_path = PathBuf::from(EXAMPLES_PATH).join("gds/test_col_inv_array.gds");
        let source_path = PathBuf::from(EXAMPLES_PATH).join("spice/col_inv_array.spice");
        let work_dir = PathBuf::from(TEST_BUILD_PATH).join("test_run_lvs_col_inv");

        assert!(
            matches!(
                run_lvs(&LvsParams {
                    work_dir: &work_dir,
                    layout_path: &layout_path,
                    layout_cell_name: "test_col_inv_array",
                    source_paths: &[source_path],
                    source_cell_name: "col_inv_array",
                    rules_dir: &PathBuf::from(SKY130_LVS),
                    rules_path: &PathBuf::from(SKY130_LVS_RULES_PATH),
                })?
                .status,
                LvsStatus::Correct
            ),
            "LVS failed"
        );
        Ok(())
    }

    #[test]
    fn test_run_lvs_fail() -> anyhow::Result<()> {
        let layout_path = PathBuf::from(EXAMPLES_PATH).join("gds/test_col_inv_array.gds");
        let source_path = PathBuf::from(EXAMPLES_PATH).join("spice/col_inv_array_incorrect.spice");
        let work_dir = PathBuf::from(TEST_BUILD_PATH).join("test_run_lvs_fail");

        assert!(
            matches!(
                run_lvs(&LvsParams {
                    work_dir: &work_dir,
                    layout_path: &layout_path,
                    layout_cell_name: "test_col_inv_array",
                    source_paths: &[source_path],
                    source_cell_name: "col_inv_array",
                    rules_dir: &PathBuf::from(SKY130_LVS),
                    rules_path: &PathBuf::from(SKY130_LVS_RULES_PATH),
                })?
                .status,
                LvsStatus::Incorrect,
            ),
            "LVS should have failed but did not"
        );
        Ok(())
    }
}
