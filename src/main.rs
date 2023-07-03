extern crate structopt;
#[macro_use]
extern crate structopt_derive;

use std::ffi::{OsStr, OsString};
use std::fs::{metadata, File, Metadata};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "the args struct",
    about = "the struct containing the args, and a description"
)]
struct Args {
    // /// Some source of some string
    // source: String,
    /// Print debug information
    #[structopt(short, long)]
    debug: bool,
    // /// Some flag pertaining to the level of something
    // #[structopt(short = "l", long = "level", default_value = "42")]
    // level: u8,
    /// Some input
    #[structopt(parse(from_os_str))]
    input: PathBuf,
    /// Some output string, guessing a file or something
    #[structopt(parse(from_os_str))]
    output: Option<PathBuf>,
    /// Where to write the output to: `stdout` or `file`
    #[structopt(short, default_value = "stdout")]
    out_type: String,
    /// File name: only required when `out-type` is set to `file`
    #[structopt(name = "FILE", required_if("out-type", "file"))]
    file_name: Option<String>,
}

fn analyze_metadata(path: &PathBuf) -> Option<Metadata> {
    match metadata(path) {
        Ok(md) => {
            let mut output_string = format!("TYPE\t{:?}", md.file_type());
            // println!("FILE TYPE\t{:?}", md.file_type());
            if md.is_dir() {
                // println!("A DIR");
                output_string.push_str("\tA DIR");
            } else if md.is_file() {
                // println!("A FILE");
                output_string.push_str("\tA FILE");
            } else if md.is_symlink() {
                // println!("A SYMLINK");
                output_string.push_str("\tA SYMLINK");
            } else {
                // println!("SOMETHING MYSTERIOUS");
                output_string.push_str("\tSOMETHING MYSTERIOUS");
            }
            output_string = format!("{}\tSIZE\t{}\n", output_string, md.len());
            println!("PERMISSIONS\t{:?}", md.permissions());
            if let Ok(time_modified) = md.modified() {
                // println!("TIME MODIFIED\t{:?}", time_last_modified);
                output_string.push_str(format!("MODIFIED\t{:?}\t", time_modified).as_str())
            }
            if let Ok(time_accessed) = md.accessed() {
                // println!("TIME ACCESSED\t{:?}", time_last_accessed);
                output_string.push_str(format!("ACCESSED\t{:?}\t", time_accessed).as_str())
            }
            if let Ok(time_created) = md.created() {
                // println!("TIME CREATED\t{:?}", time_created);
                output_string.push_str(format!("CREATED\t{:?}\n", time_created).as_str())
            }
            println!("{}", output_string);
            Some(md)
        }
        Err(e) => {
            println!("stdio error on metadata search\n{:?}", e);
            match e.raw_os_error() {
                Some(i) => println!("raw os error {}", i),
                _ => println!("couldnt get raw"),
            }
            None
        }
    }
}

fn read_if_exists(path: &PathBuf) -> (Option<Vec<u8>>, Option<usize>) {
    if let Ok(mut input_file) = File::open(path) {
        let mut input_buffer: Vec<u8> = vec![];
        let bytes_read = input_file.read_to_end(&mut input_buffer).ok();
        (Some(input_buffer), bytes_read)
    } else {
        (None, None)
    }
}

// fn write_to_file_or_create(
//     output_path: &PathBuf,
//     input_path: &PathBuf,
// ) -> (Option<File>, Option<usize>) {
//     if let Ok(mut output_file) = File::open(output_path) {
//         let bytes_written = write_subroutine(input_path, &mut output_file);
//         (Some(output_file), bytes_written)
//     } else if let Ok(mut output_file) = File::create(output_path) {
//         let bytes_written = write_subroutine(input_path, &mut output_file);
//         (Some(output_file), bytes_written)
//     } else {
//         (None, None)
//     }
// }

fn open_or_create<F>(path: &F) -> Result<File, std::io::Error>
where
    F: From<PathBuf> + std::convert::AsRef<std::path::Path> + std::convert::AsRef<std::ffi::OsStr>,
{
    if Path::new(path).exists() {
        File::open(path)
    } else {
        File::create(path)
    }
}

fn read_subroutine<R>(input: &mut R) -> Option<&mut Vec<u8>>
where
    R: Read,
{
    let mut buffer: &mut Vec<u8> = &mut vec![];
    if let Ok(x) = input.read_to_end(&mut buffer) {
        Some(buffer)
    } else {
        None
    }
}

fn read_from_file<R>(input: &mut R) -> Option<&mut Vec<u8>>
where
    R: From<PathBuf> + std::convert::AsRef<std::path::Path>,
{
    if let Ok(mut file) = File::open(&input) {
        read_subroutine(&mut file)
    } else {
        None
    }
}

fn write_from_file<R, W>(input: &mut R, output: &mut W) -> Option<Result<usize, std::io::Error>>
where
    R: From<PathBuf> + std::convert::AsRef<std::path::Path>,
    W: std::io::Write,
{
    if let Some(input_data) = read_from_file(input) {
        Some(output.write(input_data))
    } else {
        None
    }
}

#[paw::main]
fn main(args: Args) {
    // println!("From the source/value {}", args.source);
    // let _input_metadata = analyze_metadata(&args.input);
    let output = match args.output {
        Some(output_path) => {
            if let Ok(output_file) = open_or_create(&output_path) {
                write_subroutine(&mut args.input, &mut output_file)
            }
        }
        None => write_subroutine(&mut args.input, &mut std::io::stdout()),
    };
}
