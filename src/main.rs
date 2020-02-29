extern crate walkdir;
extern crate crypto;
extern crate zip;
#[macro_use]
extern crate serde_json;

use colored::*;

use std::io::{Read, Write};
use std::collections::HashMap;
use std::path::Path;
use std::fs::File;
use std::fs;
use std::time;

use crypto::digest::Digest;
use crypto::md5::Md5;

use walkdir::{WalkDir};

use zip::result::ZipError;

mod compress;


/*
 * Got file md5
 */
fn get_md5(file_path: &String) -> String {

    let mut buffer = Vec::new();
    let mut hasher = Md5::new();

    let mut f = File::open(file_path.to_owned()).unwrap();

    f.read_to_end(&mut buffer).unwrap();
    hasher.input(&buffer);

    hasher.result_str()
}

/*
 * Generate directory tree info
 */
fn path_info(path: &String) -> (HashMap<String, String>, Vec<String>) {

    let base_dir = path;
    let mut file_list: Vec<String> = vec![];
    let mut dir_list: Vec<String> = vec![];

    let mut file_dict: HashMap<String, String> = HashMap::new();

    let entrys = WalkDir::new(path);
    //for (i, entry) in entrys.into_iter().enumerate() {
    for entry in entrys {
        let entry = entry.unwrap();
        let entry_path = entry.path();
        let path = String::from(entry_path.to_str().unwrap());

        let inner_path = String::from(Path::new(&path).strip_prefix(&base_dir).unwrap().to_string_lossy());
        if entry_path.is_dir() {
            dir_list.push(inner_path.clone());
            println!(" {}                                  {}", "DIR".green(), &path);
        } else {
            file_list.push(inner_path.clone());
            let md5_str = get_md5(&path);
            file_dict.insert(md5_str.clone(), inner_path.clone());
            println!("{} {} {}", "FILE".green(), md5_str, &path);
        }
    }

    (file_dict, dir_list)
}

/*
 * Diff from dirx and diry
 * list dir or file in diry but not in dirx
 * and the file in dirx and not in diry will ignore (don't remove)
 */
fn diff_directory<'a>(dirx_info: &(HashMap<String, String>, Vec<String>),
                      diry_info: &(HashMap<String, String>, Vec<String>)) -> Vec<String> {

    let &(ref x_file_dict, ref x_dir_list) = dirx_info;
    let &(ref y_file_dict, ref y_dir_list) = diry_info;
    let mut need_packed: Vec<String> = vec![];

    for inner_dirname in y_dir_list {
        if !x_dir_list.contains(&inner_dirname) {
            need_packed.push(inner_dirname.to_owned());
            println!(" {} {}", "d".green(), &inner_dirname);
        }
    }
    for (key, value) in y_file_dict {
        let dirx_contain = x_file_dict.contains_key(key);
        if !(dirx_contain && (&x_file_dict[key] == &y_file_dict[key])) {
            need_packed.push(value.to_owned());
            if dirx_contain {
                println!(" {} {} {}", "u".yellow(), &key, value);
            } else {
                println!(" {} {} {}", "+".green(), &key, value);
            }
        }
        // else {
        //     let x_pathbuf = Path::new(&base_dirx).join(&value);
        //     let x_path = x_pathbuf.as_path();
        //     let y_pathbuf = Path::new(&base_diry).join(&value);
        //     let y_path = y_pathbuf.as_path();
        //     println!(" = {} {} {}", &key, x_path.display(), y_path.display());
        // }
    }
    need_packed
}


/*
 * Generate temp dir for package
 */
fn generate_tmpdir(diff_info: &Vec<String>, base_diry: &String, outer_dir: &String) {

    println!("\n> Collect\n");

    for p in diff_info {
        let output_pathbuf = Path::new(&outer_dir).join(&p);
        let need_create = output_pathbuf.as_path();
        let is_dir = Path::new(base_diry).join(&p).is_dir();
        if is_dir {
            println!(" {} {}", "create".green(), &need_create.display());
            fs::create_dir_all(&need_create).expect("Nope");
        } else {
            let parent_path = need_create.parent().unwrap();
            if !parent_path.exists() {
                fs::create_dir_all(&parent_path).expect("Nope");
                println!(" {} {}", "create".green(), &parent_path.display());
            }
            let from_pathbuf = Path::new(&base_diry).join(&p);
            let from = from_pathbuf.as_path();
            let to   = need_create;
            println!("   {} {}", "copy".green(), &to.display());
            fs::copy(&from, &to).expect("Nope");
        }
    }
}

/*
 * Create zip from temp package directory
 */
fn create_package(outer_dir: &str, outer_zip: &str, method: zip::CompressionMethod) -> zip::result::ZipResult<()> {
    if !Path::new(outer_dir).is_dir() {
        return Err(ZipError::FileNotFound);
    }

    let path = Path::new(outer_zip);
    let file = File::create(&path).unwrap();

    let walkdir = WalkDir::new(outer_dir.to_string());
    let it = walkdir.into_iter();

    compress::zip_dir(&mut it.filter_map(|e| e.ok()), outer_dir, file, method)?;

    Ok(())
}

/*
 * Create info json file about generated zip
 */
fn create_info_file(outer_zip: &String, info_file: &String) {

    // package.zip md5
    let mut buffer = Vec::new();
    let mut hasher = Md5::new();
    let mut f = File::open(&outer_zip).unwrap();
    f.read_to_end(&mut buffer).unwrap();
    hasher.input(&buffer);

    let metadata = fs::metadata(&outer_zip).expect("Nope");
    let file_size = metadata.len();

    // template json file
    let mut buffer = File::create(&info_file).expect("Nope");
    let info = json!({
        "md5": hasher.result_str(),
        "size": file_size,
    });
    buffer.write(info.to_string().as_bytes()).expect("Nope");
}

/*
 * eg: "dirname/" -> "dirname"
 */
fn remove_end_slash(base_dir: &str) -> String {
    if base_dir.ends_with("/") {
        base_dir[..base_dir.len()-1].to_string()
    } else {
        base_dir.to_string()
    }
}

fn main() {

    let args: Vec<_> = std::env::args().collect();
    if args.len() < 3 {
        println!("Usage: {} <dirx> <diry>", args[0]);
        std::process::exit(1);
    }

    let e = time::SystemTime::now();

    // Remove '/' end of the path
    let base_dirx: String = remove_end_slash(&args[1]);
    let base_diry: String = remove_end_slash(&args[2]);

    println!("Generate update package from {} -> {}\n", &base_dirx, &base_diry);

    // Loop for directory contain list
    let dirx_info = path_info(&base_dirx);
    let diry_info = path_info(&base_diry);


    // Generate diff info
    println!("\n> Diff Info\n");
    let diff_info = diff_directory(&dirx_info, &diry_info);

    // Copy file to temp directory
    let outer_dir = String::from("__package");
    generate_tmpdir(&diff_info, &base_diry, &outer_dir);

    // Create zip package
    if Path::new(&outer_dir).exists() {
        let outer_zip = String::from("package.zip");
        println!("\n> Create Package\n");
        match create_package(&outer_dir, &outer_zip, zip::CompressionMethod::Deflated) {
            Ok(_) => {
                println!("   {} {} written to {}", "done".green(), outer_dir, outer_zip);
                fs::remove_dir_all(&outer_dir).expect("Nope");
                let info_file = String::from("info.json");
                create_info_file(&outer_zip, &info_file);
            },
            Err(e) => println!("Error: {:?}", e),
        }
    } else {
        println!("{}", "diry has no update from dirx, maybe same".green());
    }

    let ed = time::SystemTime::now();
    println!("\ntime spend: {:?}", ed.duration_since(e).unwrap());
    println!("{}", "Success!".green().bold());
}



