use auto_lang::config::AutoConfig;
use auto_val::{AutoPath, AutoResult, AutoStr};
use colored::Colorize;
use dialoguer::{console::Style, theme::ColorfulTheme, Input, Select};
use reqwest::blocking::get;
use std::fs::{self, File};
use std::path::Path;
use std::sync::OnceLock;
use std::{cmp::Ordering, env};
use version_compare::{CompOp, VersionCompare};
use zip::ZipArchive;

static MY_THEME: OnceLock<ColorfulTheme> = OnceLock::new();

pub struct ReleaseInfo {
    pub name: AutoStr,
    pub version: AutoStr,
    pub url: AutoStr,
}

fn list_releases(name: &AutoStr) -> AutoResult<ReleaseInfo> {
    let url = format!(
        "https://gitee.com/auto-stack/auto-man/raw/master/config/{}.at",
        name
    );

    println!("checking releases from {}", url);

    let page = get(url)?.text()?;

    let config = AutoConfig::new(page)?;

    let releases = config.root.get_nodes("release");

    let mut versions = vec![];
    for r in releases.iter() {
        println!("Release: {}", r);
        let v = r.get_prop("version").to_astr_or("unknown");
        versions.push(v);
    }
    versions.sort_by(|a, b| match VersionCompare::compare(a, b) {
        Ok(CompOp::Eq) => Ordering::Equal,
        Ok(CompOp::Lt) => Ordering::Greater,
        Ok(CompOp::Gt) => Ordering::Less,
        Err(_) => Ordering::Equal,
        _ => Ordering::Equal,
    });

    // select one version
    //
    println!();
    let selection = Select::with_theme(MY_THEME.get().unwrap())
        .with_prompt("Which version do you want to install?")
        .default(0)
        .items(&versions)
        .interact()?;

    let version = versions[selection].clone();
    println!("Selected version: {}", version);

    let mut url = AutoStr::new();
    for r in &releases {
        let v = r.get_prop("version").to_astr_or("unknown");
        if v == version {
            url = r.get_prop("url").to_astr_or("unknown").clone();
        }
    }
    if url == "unknown" || url.is_empty() {
        println!("No URL found for the selected version: {}", version);
    }

    Ok(ReleaseInfo {
        name: name.clone(),
        version,
        url: url.into(),
    })
}

fn get_install_dir(tool: &str) -> AutoResult<AutoStr> {
    println!("Getting last installation directory for {}", tool);
    let home = dirs::home_dir().ok_or("Can't open home dir")?;
    let auto_dir = home.join(".auto");
    // if au.at not in dir, this is the first time to install
    let au_conf = auto_dir.join("auto-up.at");
    if au_conf.is_file() {
        // look for installation dir
        let conf = AutoConfig::read(&au_conf)?;
        let install_dir = conf.root.get_prop_of("auto-man").to_astr();
        println!("Last installation directory for {}: {}", tool, install_dir);
        return Ok(install_dir);
    } else {
        println!("Not found.");
        // first time
        // let user input one dir
        let default_am_location = auto_dir.join(tool);
        println!();
        let input = Input::<String>::with_theme(MY_THEME.get().unwrap())
            .with_prompt("Enter installation directory")
            .with_initial_text(default_am_location.to_str().unwrap())
            .interact_text()?;

        // store this dir to auto-up.at
        fs::create_dir_all(default_am_location)?;
        let code = format!("\"auto-man\": \"{}\"\n", input);
        fs::write(au_conf, code)?;
        return Ok(input.into());
    }
}

fn auto_up() -> AutoResult<()> {
    // 1. list all releases and let user select one
    let tool = "auto-man".into();
    let info = list_releases(&tool)?;
    // check installation directory
    let install_dir = get_install_dir(&tool)?;
    // download installation zip to temp dir
    let temp_dir = env::temp_dir();

    println!(
        "downloading {} to temp dir {}",
        info.name,
        temp_dir.display()
    );

    // download zip
    let zip_name = info.url.split('/').last().unwrap();
    let zip_path = temp_dir.join(zip_name);
    if zip_path.is_file() {
        println!(
            "zip file {} already exists, deleting...",
            zip_path.to_str().unwrap()
        );
        std::fs::remove_file(zip_path.as_path())?;
    }
    println!("Downloading {}...", info.url);
    let resp = reqwest::blocking::get(info.url.as_str()).expect("request failed");
    let body = resp.bytes().expect("body invalid");
    std::fs::write(zip_path.clone(), &body)?;

    println!("Download successful, installing...");
    let install_path = Path::new(install_dir.as_str());

    // unzip
    extract_zip(&zip_path, &info, &install_path)?;

    println!("Installation directory: {}", install_dir);

    // write am.at
    write_am_at(&install_path, &info.name)?;

    // clear index dirs
    let index_dir = install_path.join("index");
    if index_dir.is_dir() {
        std::fs::remove_dir_all(&index_dir)?;
    }
    println!("Deleting index directory: {}", index_dir.display());

    // 2. download selected release
    // 3. install selected release
    println!("OK");
    Ok(())
}

fn write_am_at(install_path: &Path, _name: &AutoStr) -> AutoResult<()> {
    // let user select which index base to use:
    // let index_bases = vec!["default", "soutek", "boe"];
    // let selection = Select::with_theme(MY_THEME.get().unwrap())
    //     .with_prompt("Which index base do you want to use?")
    //     .default(0)
    //     .items(&index_bases)
    //     .interact()?;
    // let selection = 0;

    // let index_urls = vec![
    // "git@gitee.com:auto-stack/auto-index.git",
    // "git@codeup.aliyun.com:soutek/auto-stack/auto-index.git",
    // "ssh://git@gitlab.boevxa.lan:2424/ds/platform/boevxabafdemo/gitlabindex.git",
    // ];

    // let url = if selection == 2 {
    //     let input = Input::<String>::with_theme(MY_THEME.get().unwrap())
    //         .with_prompt("Enter the index base URL:")
    //         .interact_text()?;
    //     input
    // } else if selection <= 1 {
    // let name = if selection <= 1 {
    // index_urls[selection].to_string()
    // } else {
    // panic!("Invalid selection");
    // };

    // write to am.at
    let am_at_file = install_path.join("am.at");
    let am_url = "https://gitee.com/auto-stack/auto-man/raw/master/config/am.at";
    let page = get(am_url)?.text()?;
    let home_dir = dirs::home_dir().unwrap();
    let home_dir = home_dir.to_str().unwrap();
    println!("page:{}", page);
    let page = page.replace("${HOME}", home_dir);
    println!("page:{}", page);
    std::fs::write(&am_at_file, page)?;

    // let mut w = BufWriter::new(am_at_file);
    // writeln!(w, "// index bases")?;
    // writeln!(w, "index: {{")?;
    // for i in 0..index_bases.len() {
    //     writeln!(w, "    {}: \"{}\"", index_bases[i], index_urls[i])?;
    // }
    // writeln!(w, "}}\n")?;
    // writeln!(w, "// default index")?;
    // writeln!(w, "default_index: \"{}\"\n", name)?;
    // writeln!(w, "// location to store automan related configurations")?;
    // let install_dir = install_path.to_str().unwrap();
    // writeln!(w, "am: \"{}\"", install_dir)?;
    println!("am.at written to {}", am_at_file.display());
    Ok(())
}

fn get_simple_name(name: &AutoStr) -> AutoStr {
    match name.as_str() {
        "auto-man" => "am".into(),
        "auto-gen" => "ag".into(),
        _ => name.clone(),
    }
}

fn extract_zip(zip_path: &Path, info: &ReleaseInfo, install_dir: &Path) -> AutoResult<()> {
    std::fs::create_dir_all(install_dir)?;
    let mut zip = ZipArchive::new(File::open(zip_path)?)?;
    for i in 0..zip.len() {
        let mut file = zip.by_index(i)?;
        let basename = file.name();
        let name = info.name.to_string();
        let versioned_path = install_dir.join(name.clone() + "_" + info.version.as_str() + ".exe");
        let simple_name = get_simple_name(&info.name);
        let link_path = install_dir.join(simple_name.to_string() + ".exe");
        if basename != name + ".exe" {
            continue;
        }
        println!("extracting {}", basename);
        if file.is_dir() {
            std::fs::create_dir_all(&versioned_path)?;
        } else {
            let mut out = File::create(&versioned_path)?;
            std::io::copy(&mut file, &mut out)?;
            println!(
                "{}",
                format!("installed {}", versioned_path.display()).bright_blue()
            );

            let exe_path = AutoPath::from(versioned_path);
            let link_path = AutoPath::from(link_path);
            link_exe(&exe_path, &link_path)?;
            println!(
                "{}",
                format!("linked {}", link_path.to_astr()).bright_blue()
            );
        }
    }
    Ok(())
}

pub fn link_exe(exe_path: &AutoPath, link_path: &AutoPath) -> AutoResult<()> {
    let abs_link = link_path.abs();
    println!("abs_link: {}", abs_link);
    let link_exists = link_path.is_file();
    // rename old link to backup
    let back = format!("{}.bak", abs_link);
    if link_exists {
        std::fs::rename(link_path.path(), back.clone()).unwrap();
    }

    // create new link
    println!("trying to link {} to {}", exe_path, link_path);
    std::fs::hard_link(exe_path.path(), link_path.path()).unwrap();

    if link_exists {
        // delete backup
        std::fs::remove_file(back).unwrap();
    }
    Ok(())
}

pub fn upgrade() -> AutoResult<()> {
    std::thread::spawn(|| {
        MY_THEME.get_or_init(|| ColorfulTheme {
            prompt_style: Style::default().bold().yellow().bright(),
            ..Default::default()
        });
    })
    .join()
    .unwrap();

    auto_up()?;
    Ok(())
}
