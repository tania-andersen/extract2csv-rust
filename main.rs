use regex::Regex;
use std::ffi::OsStr;
use std::io;
use std::path::{Path, PathBuf};
use std::{env, fs};

const DEFAULT_OUT_FILENAME: &str = "out.csv";

fn main() -> io::Result<()> {
    let mut args: Vec<String> = env::args().collect();
    for i in 0..args.len() {
        // Prefix regex expression with (?ms) for "multiline" and "dot matches all".
        let mut prefix = "(?ms)".to_owned();
        args[i] = args[i].replace("*", "(.*?)");
        prefix.push_str(&args[i]);
        args[i] = prefix;
    }
    let dir = env::current_dir()?;
    let mut files_in_dir: Vec<PathBuf> = Vec::new();
    for file in fs::read_dir(dir)? {
        let file = file?;
        let pathbuf = file.path();
        if pathbuf.extension().unwrap_or(OsStr::new("")) == "txt" {
            files_in_dir.push(pathbuf);
        }
    }
    let mut csv_wtr = csv::Writer::from_path(Path::new(DEFAULT_OUT_FILENAME))?;
    let mut patterns: Vec<Regex> = Vec::new();
    for pattern_string in &args[1..args.len()] {
        let pattern = Regex::new(&pattern_string).unwrap();
        patterns.push(pattern)
    }
    for filepath in files_in_dir {
        let mut found: Vec<Vec<String>> = Vec::new();
        let filename = &filepath.file_name().unwrap().to_string_lossy().into_owned();
        let text = fs::read_to_string(filepath)?;
        for pattern in &patterns {
            let matches = pattern.captures_iter(&text);
            let mut found_matches: Vec<String> = Vec::new();
            for mat in matches {
                for i in 1..mat.len() {
                    let group = &mat[i];
                    let group_text = group.trim();
                    found_matches.push(group_text.to_owned());
                }
            }
            found.push(found_matches);
        }
        let max_matches = found.iter().map(|v| v.len()).max().unwrap_or(0);
        for row in 0..max_matches {
            csv_wtr.write_field(filename)?;
            for column in 0..patterns.len() {
                // found[0] contains matches for pattern 0, found[1] contains matches for pattern 1...
                let mat = &found[column];
                if mat.len() > row {
                    let cell = &mat[row];
                    csv_wtr.write_field(cell)?;
                } else {
                    csv_wtr.write_field("")?;
                }
            }
            csv_wtr.write_record(None::<&[u8]>)?;
        }
    }
    csv_wtr.flush()?;
    Ok(())
}

#[test]
fn simple_usecase_test() {
    let here = env::current_dir();
    let here = match here {
        Ok(dir) => dir,
        Err(error) => panic!("Cannot get dir: {:?}", error),
    };
    println!("You are here: {:?}", here);
    let copy_ok = fs::copy("target\\debug\\extract2csv.exe", "tests\\extract2csv.exe");
    match copy_ok {
        Ok(_) => (),
        Err(error) => panic!("Cannot copy program to test folder: {:?}", error),
    }
    let cd_ok = env::set_current_dir("tests");
    match cd_ok {
        Ok(_) => (),
        Err(error) => panic!("Cannot change directory to tests folder: {:?}", error),
    }
    let output = std::process::Command::new(".\\extract2csv.exe")
        .arg("Tidspunkt for indberetningen*Referencenummer")
        .arg("virksomhedens navn*Afdeling")
        .arg("Beskriv hændelsen*Hvor fandt hændelsen fysisk sted")
        .output();
    match output {
        Ok(_) => (),
        Err(error) => panic!("Cannot run program: {:?}", error),
    };
    let file_content = fs::read_to_string(DEFAULT_OUT_FILENAME);
    let file_content = match file_content {
        Ok(file_content) => file_content,
        Err(error) => panic!("Cannot open output file: {:?}", error),
    };
    assert!(
        file_content.contains("2019-442-2007.PDF.txt"),
        "Missing field"
    );
    assert!(
        file_content.contains("onsdag november 6, 2019 14:14:01"),
        "Missing field"
    );
    assert!(file_content.contains("Kalundborg kommune"), "Missing field");
    assert!(
        file_content.contains("hvis denne kode er 3"),
        "Missing field"
    );
    assert!(
        file_content.contains("torsdag november 7, 2019 09:32:32"),
        "Missing field"
    );
    assert!(
        file_content.contains("fredag november 15, 2019 08:27:28"),
        "Missing field"
    );
    assert!(
        file_content.contains("fredag november 8, 2019 11:56:09"),
        "Missing field"
    );
    assert!(
        file_content.contains("fredag november 8, 2019 13:08:49"),
        "Missing field"
    );
    assert!(
        file_content.contains("koden betyder, at billedet ikke må vises"),
        "Missing field"
    );
    assert!(
        file_content.contains("Kun ændring i antal berørte"),
        "Missing field"
    );
}
