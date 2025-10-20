use anyhow::{Context, Result};
use clap::Parser;
use image::ImageReader; // Fixed: Use direct image::ImageReader (no io::Reader alias)
use oxipng::{optimize_from_memory, Options};
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

    /// Optimize PNGs post-zap (lossless compression, smaller files)
    #[arg(short = 'z', long, default_value_t = false)] // Fixed: -z, not -o
    optimize: bool,

    /// Backup originals with .bak suffix (for in-place runs)
    #[arg(short = 'b', long, default_value_t = false)]
    backup: bool,
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
                && extensions
                    .iter()
                    .any(|ext| e.path().extension().and_then(|s| s.to_str()) == Some(ext))
        });

    for entry in walker {
        let src_path = entry.path();
        let file_name = src_path.file_name().unwrap().to_str().unwrap();
        let ext = src_path.extension().unwrap().to_str().unwrap();

        // Determine if in-place mode
        let is_inplace = args.output.is_none();

        // Handle backup in in-place mode (before dry-run)
        if is_inplace && args.backup {
            let mut backup_path = src_path.to_path_buf();
            if let Some(e) = backup_path.extension() {
                // Fix E0277: Convert OsStr to &str for formatting
                let ext_str = e.to_str().unwrap_or("");
                let new_ext = format!("bak.{}", ext_str);
                backup_path.set_extension(new_ext);

                if !args.dry_run {
                    fs::copy(src_path, &backup_path).with_context(|| {
                        format!("Failed to create backup for {}", src_path.display())
                    })?;
                    println!("  └─ Backed up to: {}", backup_path.display());
                } else {
                    // Fix E0425: backup_path now in scope for dry-run
                    println!("  └─ Would backup to: {}", backup_path.display());
                }
            }
        }

        // Compute final dest_path
        let dest_path = if let Some(out_dir) = &args.output {
            out_dir.join(file_name)
        } else {
            src_path.to_path_buf()
        };

        if args.dry_run {
            println!(
                "Would process: {} -> {}",
                src_path.display(),
                dest_path.display()
            );
            processed += 1;
            continue;
        }

        match process_image(src_path, &dest_path, ext, args.optimize) {
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

    println!(
        "\nSummary: {} processed, {} skipped, {} errors",
        processed, skipped, errors
    );

    if errors > 0 {
        std::process::exit(1);
    }

    Ok(())
}

fn process_image(src: &Path, dest: &Path, ext: &str, optimize: bool) -> Result<()> {
    let img = ImageReader::open(src)?.decode()?;
    img.save(dest)
        .with_context(|| format!("Failed to save {}", ext.to_uppercase()))?;

    if optimize && ext.to_lowercase() == "png" {
        let data = fs::read(dest)?;
        let opts = Options::from_preset(2);
        let optimized = optimize_from_memory(&data, &opts)?;
        fs::write(dest, &optimized)?;
    }

    Ok(())
}
