use csv::Writer;
use petgraph::dot::{Config, Dot};
use petgraph::graph::DiGraph;
use petgraph::matrix_graph::node_index;
use petgraph::stable_graph::NodeIndex;
use regex::Regex;
use serde::Serialize;
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

// Define a struct for your CSV rows
#[derive(Serialize)]
struct Edge {
    src: String,
    dst: String,
}

// use petgraph::
fn a_code_to_usize(acode: &str) -> usize {
    acode[1..].parse().unwrap()
}

// filename to acode
fn filename_to_acode(filepath: &str) -> String {
    Path::new(filepath)
        .file_stem() // Extracts the file stem (filename without extension)
        .and_then(|stem| stem.to_str())
        .unwrap_or_default()
        .to_string()
}

fn get_available_acodes(directory_path: &Path) -> Vec<String> {
    let paths = fs::read_dir(directory_path).expect("Could not read directory");
    let mut available_acodes = vec![];

    for path in paths {
        if let Ok(entry) = path {
            let filename = entry.file_name().into_string().unwrap();
            if let Some(acode) = filename.split('.').next() {
                available_acodes.push(acode.to_string());
                // available_acodes.insert(acode.to_string());
            }
        }
    }

    available_acodes
}

fn process_file(file_path: &Path, available_acodes: &Vec<String>) -> HashSet<String> {
    let content = fs::read_to_string(file_path).expect("Something went wrong reading the file");

    let main_id = extract_main_id(&content);
    extract_acodes(&content, available_acodes)
}

fn extract_main_id(content: &str) -> String {
    let main_id_regex = Regex::new(r"%I (A\d{6})").unwrap();
    main_id_regex
        .captures(content)
        .and_then(|cap| cap.get(1).map(|match_| match_.as_str().to_string()))
        .unwrap_or_default()
}

fn extract_acodes(content: &str, available_acodes: &Vec<String>) -> HashSet<String> {
    let all_acode_regex = Regex::new(r"A\d{6}").unwrap();
    all_acode_regex
        .find_iter(content)
        .map(|match_| match_.as_str().to_string())
        .filter(|acode| available_acodes.contains(acode))
        .collect()
}

fn get_subdirectories(base_path: &Path) -> Vec<PathBuf> {
    fs::read_dir(base_path)
        .expect("Could not read base directory")
        .filter_map(|entry| {
            let entry = entry.ok()?;
            if entry.metadata().ok()?.is_dir() {
                Some(entry.path())
            } else {
                None
            }
        })
        .collect()
}

// get_seqs_from_dir
fn get_seqs_from_dir(directory_path: &Path) -> Vec<String> {
    fs::read_dir(directory_path)
        .expect("Could not read directory")
        .filter_map(|entry| entry.ok()) // Filter out any Err results and unwrap Ok values
        .filter_map(|entry| entry.path().file_name()?.to_str().map(String::from)) // Convert OsStr to String
        .collect::<Vec<String>>() // Collect into Vec<String>
}

fn get_all_seq_filenames(base_directory: &Path) -> Vec<String> {
    let mut all_filepaths = Vec::new();

    let subdirectories = fs::read_dir(base_directory)
        .expect("Could not read base directory")
        .filter_map(Result::ok)
        .filter(|entry| entry.metadata().map(|m| m.is_dir()).unwrap_or(false));

    for subdir in subdirectories {
        let subdir_path = subdir.path();
        let seq_files = fs::read_dir(subdir_path.clone())
            .expect("Could not read subdirectory")
            .filter_map(Result::ok)
            .filter(|entry| entry.path().extension().and_then(|ext| ext.to_str()) == Some("seq"))
            .map(|entry| {
                subdir_path
                    .join(entry.path())
                    .to_string_lossy()
                    .into_owned()
            });

        all_filepaths.extend(seq_files);
    }

    all_filepaths
}

fn make_graph(fns: Vec<String>) {
    let mut g = DiGraph::<usize, ()>::new();
    let available_acodes = fns
        .iter()
        .map(|filename| filename_to_acode(&filename))
        .collect();

    let mut acode_usize_to_index = std::collections::HashMap::new();
    // g.
    for filename in fns.iter() {
        let acode = filename_to_acode(&filename);
        let v = a_code_to_usize(&acode);
        let vid = g.add_node(v);
        acode_usize_to_index.insert(v, vid);
        // assert_eq!(v, vid.index() + 1);
    }
    // g.add_node(());

    for (i, filename) in fns.iter().enumerate() {
        let acode = filename_to_acode(&filename);
        let v = a_code_to_usize(&acode);
        let vid = acode_usize_to_index[&v];
        let out_neighbors = process_file(Path::new(filename), &available_acodes);
        for neighbor in out_neighbors {
            let u = a_code_to_usize(&neighbor);
            let uid = acode_usize_to_index[&u];
            if u == v {
                continue;
            }

            g.add_edge(vid, uid, ());
        }
        if i == 5000 {
            let mut file = File::create("graph_big.dot").expect("Unable to create file");

            println!("Processed {} files", i);
            writeln!(
                file,
                "{:?}",
                Dot::with_config(&g, &[Config::EdgeNoLabel, Config::NodeIndexLabel])
            )
            .expect("Unable to write to file");
            panic!();
        }
    }
    // println!("{:?}", Dot::with_config(&g, &[Config::EdgeNoLabel]));

    // Write the Dot representation of the graph to the file
}

fn write_to_csv(fns: Vec<String>, output_file: &Path) {
    let available_acodes = fns.iter().map(|x| filename_to_acode(x)).collect();

    let mut wtr = Writer::from_path(output_file).expect("Unable to create CSV writer");

    for filename in fns.iter() {
        let src_acode = filename_to_acode(filename);
        let out_neighbors = process_file(Path::new(filename), &available_acodes);

        for dst_acode in out_neighbors {
            if src_acode == dst_acode {
                continue;
            }
            let edge = Edge {
                src: src_acode.clone(),
                dst: dst_acode,
            };

            wtr.serialize(edge).expect("Unable to write row to CSV");
        }
    }

    wtr.flush().expect("Unable to flush CSV writer");
}

fn main() {
    let dir = Path::new(r"C:\Users\anand\src\oeisdata\seq");
    let dirs = get_subdirectories(dir);
    println!("{:?}", dirs);
    let fns = get_all_seq_filenames(dir);
    // print len
    println!("{:?}", fns.len());
    //print first
    println!("{:?}", fns[0]);

    let aco = filename_to_acode(&fns[0]);
    println!("{:?}", aco);

    let output_file = Path::new("g.csv");
    write_to_csv(fns, output_file);
    // make_graph(fns);
}
