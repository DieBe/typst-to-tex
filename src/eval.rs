use std::{collections::HashMap, process::Stdio};

use camino::Utf8Path;
use color_eyre::eyre::{bail, Context as _, Result};

pub fn run_eval(main_file: &Utf8Path) -> Result<HashMap<String, String>> {
    let query_result = std::process::Command::new("typst")
        .arg("eval")
        .arg("query(<ttt-state>).map(it => it.value)")
        .arg("--in")
        .arg(main_file)
        .stderr(Stdio::inherit())
        .output()
        .with_context(|| "Failed to run `typst eval`")?;

    if !query_result.status.success() {
        bail!("Typst eval failed")
    }

    let result_s = &query_result.stdout;
    let result =
        facet_json::from_slice::<Vec<HashMap<String, String>>>(result_s).with_context(|| {
            format!(
                "Failed to decode result of typst eval as a Vec of hash maps.\nRaw result: {}",
                String::from_utf8_lossy(result_s)
            )
        })?;

    match result.as_slice() {
        [] => {
            println!("typst eval did not find anything marked <ttt-state>. Did you forget to `#ttt-eval.emit`?");
            Ok(Default::default())
        }
        [thing] => Ok(thing.clone()),
        [.., thing] => {
            println!("Fonud multiple metadatas marked <ttt-state>. Defaulting to the last one");
            Ok(thing.clone())
        }
    }
}
