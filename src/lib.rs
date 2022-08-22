#![deny(clippy::all)]

use std::path::Path;
use substring::Substring;
use tsconfig::{CompilerOptions, TsConfig};
use cached::proc_macro::{cached, once};

mod node_resolve;
use std::{env::current_dir, path::PathBuf};

#[macro_use]
extern crate napi_derive;
/** Remove any trailing querystring from module id. */
fn remove_query_string(id: String) -> String {
  let query_string_index = id.find('?');
  if query_string_index.is_some() {
    return String::from(id.substring(0, query_string_index.unwrap()));
  }
  return id;
}

// Read tsConfig paths
#[cached]
fn get_ts_config_path(ts_config_file: String) -> PathBuf {
  if ts_config_file.starts_with('/') {
    if ts_config_file.ends_with(".json") {
      Path::new(ts_config_file.as_str()).to_path_buf()
    } else {
      Path::new(ts_config_file.as_str())
        .join("tsconfig.json")
        .to_path_buf()
    }
  } else {
    Path::new(current_dir().unwrap().to_str().unwrap()).join(ts_config_file)
  }
}

#[once(time=10, sync_writes = true)]
fn get_ts_config(ts_config_file: String) -> Result<TsConfig, String> {
  // Read tsConfig paths
  let tsconfig_path = get_ts_config_path(ts_config_file);

  let config = TsConfig::parse_file(&tsconfig_path);
  if config.is_ok() {
    return Ok(config.unwrap());
  }
  Err(String::from("No tsConfig file found"))
}

// Get base dir to search for
// 1. if no tsconfig file found. return current work dir
// 2. if no baseUrl listed in tsconfig. return the tsconfig file directory
// 3. if baseUrl is present. join baseUrl with tsconfig file directory as base dir
#[cached]
fn get_base_dir(ts_config_file: String) -> PathBuf {
  let ts_config = get_ts_config(ts_config_file.clone());

  // if no config file found
  if ts_config.is_err() {
    return current_dir().ok().unwrap();
  }

  let compiler_options = ts_config.clone().unwrap().to_owned().compiler_options.clone();

  let ts_config_dir = get_ts_config_path(ts_config_file.clone())
    .parent()
    .unwrap()
    .to_path_buf();
  // use tsconfig file path as base dir when no baseDir or no compiler options
  if compiler_options.clone().is_none()
    || compiler_options.clone()
      .and_then(|options| options.base_url)
      .is_none()
  {
    return ts_config_dir;
  }

  let base_url = compiler_options.unwrap().base_url.unwrap();
  ts_config_dir.join(base_url)
}

#[napi_derive::napi(object)]
pub struct ResolveResult {
  pub found: bool,
  pub path: String,
}

// TODO: Implement package export syntax
#[napi]
pub fn resolve(source_input: String, file: String, ts_config_file: String) -> ResolveResult {
  // Remove query string
  let source = remove_query_string(source_input);

  // let base_dir = tsconfig_path.parent().unwrap().to_path_buf();
  let base_dir = get_base_dir(ts_config_file.clone());

  // Start resolve normal paths
  let resolver = node_resolve::Resolver::new()
    .with_extensions(&[
      String::from(".js"),
      String::from(".json"),
      String::from(".node"),
      String::from(".mjs"),
      String::from(".cjs"),
      String::from(".ts"),
      String::from(".tsx"),
      String::from(".d.ts"),
    ])
    .with_basedir(base_dir.to_path_buf())
    .with_main_fields(&[
      String::from("types"),
      String::from("typings"),
      // APF: https://angular.io/guide/angular-package-format
      String::from("fesm2020"),
      String::from("fesm2015"),
      String::from("esm2020"),
      String::from("es2020"),
      String::from("module"),
      String::from("jsnext:main"),
      String::from("main"),
    ]);

  let mut resolved;
  if file.starts_with("/") {
    let base_dir = PathBuf::from(file).parent().unwrap().to_path_buf();

    if !source.starts_with('.') {
      resolved = resolver.resolve(source.as_str());
    } else {
      resolved = resolver.with_basedir(base_dir).resolve(source.as_str());
    }
  } else {
    resolved = resolver
      .with_basedir(base_dir.to_path_buf())
      .resolve(source.as_str());
  }

  if resolved.is_ok() {
    return ResolveResult {
      found: true,
      path: String::from(resolved.ok().unwrap().to_str().unwrap()),
    };
  }

  resolved = resolver
    .with_basedir(base_dir.to_path_buf())
    .resolve(format!("@types/{}", source.as_str()).as_str());

  if resolved.is_ok() {
    return ResolveResult {
      found: true,
      path: String::from(resolved.ok().unwrap().to_str().unwrap()),
    };
  }

  let paths_map = get_ts_config(ts_config_file.clone().to_string())
    .ok()
    .and_then(|config| config.compiler_options)
    .and_then(|option| option.paths);

  if paths_map.is_none() {
    return ResolveResult {
      found: false,
      path: String::from(""),
    };
  }

  // Iter paths to do full path match
  for (path_pattern, dest_paths) in paths_map.unwrap() {
    let star_match = node_resolve::match_star(path_pattern.clone(), source.clone());

    if star_match.is_err() {
      continue;
    }

    for dest_path in dest_paths.iter() {
      let physical_path = dest_path.replace("*", star_match.clone().unwrap().as_str());
      resolved = resolver.with_basedir(base_dir.clone()).resolve(
        base_dir
          .join(physical_path.clone())
          .to_path_buf()
          .to_str()
          .unwrap(),
      );

      if resolved.is_ok() {
        return ResolveResult {
          found: true,
          path: String::from(resolved.ok().unwrap().to_str().unwrap()),
        };
      }
    }
  }

  ResolveResult {
    found: false,
    path: String::from(""),
  }
}
