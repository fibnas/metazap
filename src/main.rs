use anyhow::{Context, Result};
use clap::Parser;
use image::ImageReader;  // Fixed: Use direct image::ImageReader (no io::Reader alias)
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(author, version, about = "Zap metadata from PNG/JPG images in a directory", long_about = None)]
struct Args {
    /// Input directory (default: current dir)
    #[arg(short, long, default_value = ".")]
    input: PathBuf,

    /// Output directory (default: overwrite in-place)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Recurse into subdirectories
    #[arg(short, long, default_value_t = true)]
    recursive: bool,

    /// Dry run: show what would be done, no changes
    #[arg(short, long, default_value_t = false)]
    dry_run: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Fixed: Use ! instead of .not()
    if !args.input.exists() {
        anyhow::bail!("Input directory '{}' does not exist", args.input.display());
    }

    // Fixed: Use unwrap_or to get clean &PathBuf (avoids &&PathBuf mess)
    let output_dir = args.output.as_ref().unwrap_or(&args.input);
    // Fixed: Use ! instead of .not()
    if !output_dir.exists() && !args.dry_run {
        fs::create_dir_all(output_dir).context("Failed to create output directory")?;
    }

    let extensions: Vec<&str> = vec!["png", "jpg", "jpeg"];
    let mut processed = 0;
    let skipped = 0;
    let mut errors = 0;

    let walker = WalkDir::new(&args.input)
        .max_depth(if args.recursive { std::usize::MAX } else { 1 })
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file()
                && extensions.iter().any(|ext| {
                    e.path().extension().and_then(|s| s.to_str()) == Some(ext)
                })
        });

    for entry in walker {
        let src_path = entry.path();
        let file_name = src_path.file_name().unwrap().to_str().unwrap();
        let ext = src_path.extension().unwrap().to_str().unwrap();

        // Fixed: Now output_dir is &PathBuf, so direct == works (coerces via Deref)
        let dest_path = if output_dir == &args.input {
            src_path.to_path_buf()
        } else {
            output_dir.join(file_name)
        };

        if args.dry_run {
            println!("Would process: {} -> {}", src_path.display(), dest_path.display());
            processed += 1;
            continue;
        }

        match process_image(src_path, &dest_path, ext) {
            Ok(_) => {
                println!("Zapped: {} -> {}", src_path.display(), dest_path.display());
                processed += 1;
            }
            Err(e) => {
                eprintln!("Error zapping {}: {}", src_path.display(), e);
                errors += 1;
            }
        }
    }

    println!("\nSummary: {} processed, {} skipped, {} errors", processed, skipped, errors);

    if errors > 0 {
        std::process::exit(1);
    }

    Ok(())
}

fn process_image(src: &Path, dest: &Path, ext: &str) -> Result<()> {
    let img = ImageReader::open(src)?.decode()?;  // Pixels only, metadata zapped

    // One call: auto-format from dest.ext(), with sensible defaults
    img.save(dest).with_context(|| format!("Failed to save {}", ext.to_uppercase()))?;

    Ok(())
}
