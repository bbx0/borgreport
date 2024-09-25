// SPDX-FileCopyrightText: 2024 Philipp Micheel <bbx0+borgreport@bitdevs.de>
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::{Context, Result};
use clap::ValueEnum;
use clap_complete::{generate_to, Shell};
use clap_mangen::Man;
use std::path::Path;

#[allow(dead_code)]
#[path = "src/cli.rs"]
mod cli;

fn build_shell_completions(out_dir: &Path) -> Result<()> {
    for &shell in Shell::value_variants() {
        generate_to(
            shell,
            &mut cli::command(),
            std::env::var("CARGO_PKG_NAME").context("CARGO_PKG_NAME not defined!")?,
            out_dir,
        )?;
    }

    Ok(())
}

// The real manpage is build via help2man. This is to keep track of clap_mangen improvements.
fn build_manpages(out_dir: &Path) -> Result<std::path::PathBuf, std::io::Error> {
    Man::new(cli::command()).generate_to(out_dir)
}

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=src/cli.rs");
    // Create `target/(release|debug)/assets/` folder.
    let mut asset_dir =
        std::path::PathBuf::from(std::env::var_os("OUT_DIR").context("OUT_DIR not defined!")?)
            .ancestors()
            .nth(3)
            .context("Cannot navigate 3 level up in given OUT_DIR!")?
            .to_path_buf();
    asset_dir.push("assets");
    std::fs::create_dir_all(&asset_dir)?;

    let mut man_dir = asset_dir.clone();
    man_dir.push("man");
    std::fs::create_dir_all(&man_dir)?;
    build_manpages(&man_dir)?;

    let mut shell_dir = asset_dir.clone();
    shell_dir.push("shell_completions");
    std::fs::create_dir_all(&shell_dir)?;
    build_shell_completions(&shell_dir)?;

    Ok(())
}
