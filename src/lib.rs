#![deny(clippy::all)]

use std::path::Path;
use substring::Substring;
use tsconfig::{CompilerOptions, TsConfig};

mod node_resolve;
use std::{env::current_dir, path::PathBuf};

#[macro_use]
extern crate napi_derive;

// Is source path match the tsConfig pattern
fn match_star(pattern: String, search: String) -> Result<String, String> {
  if search.len() < pattern.len() {
    return Err(String::from(""));
  }

  if pattern == "*" {
    return Ok(search);
  }

  if search == pattern {
    return Ok(String::from(""));
  }

  let star_index = pattern.find("*");
  if star_index.is_none() {
    return Err(String::from(""));
  }

  let part1 = pattern.substring(0, star_index.unwrap());
  let part2 = pattern.substring(star_index.unwrap() + 1, pattern.len());

  if search.substring(0, star_index.unwrap()) != part1 {
    return Err(String::from(""));
  }

  if search.substring(search.len() - part2.len(), search.len()) != part2 {
    return Err(String::from(""));
  }

  Ok(String::from(
    search.substring(star_index.unwrap(), search.len() - part2.len()),
  ))
}

/** Remove any trailing querystring from module id. */
fn remove_query_string(id: String) -> String {
  let query_string_index = id.find('?');
  if query_string_index.is_some() {
    return String::from(id.substring(0, query_string_index.unwrap()));
  }
  return id;
}

#[napi_derive::napi(object)]
pub struct ResolveResult {
  pub found: bool,
  pub path: String,
}

#[napi]
pub fn resolve(source_input: String, file: String, ts_config_file: String) -> ResolveResult {

  // Remove query string
  let source = remove_query_string(source_input);
  
  // Read tsConfig paths
  let tsconfig_path = if ts_config_file.starts_with('/') {
    if ts_config_file.ends_with(".json") {
      Path::new(ts_config_file.as_str()).to_path_buf()
    } else {
      Path::new(ts_config_file.as_str())
        .join("tsconfig.json")
        .to_path_buf()
    }
  } else {
    Path::new(current_dir().unwrap().to_str().unwrap()).join(ts_config_file)
  };

  let config = TsConfig::parse_file(&tsconfig_path).unwrap();

  // Get base dir for tsconfig
  // let base_url = config
  //   .compiler_options
  //   .and_then(|o| o.base_url.and_then(|base_url| Some(base_url)))
  //   .unwrap_or(String::from("."))
  //   .clone();
  // let base_url_str = base_url.as_str();
  // TODO: implement base url
  let base_dir = tsconfig_path.parent().unwrap().to_path_buf();

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

  print!("try types {}", resolved.is_ok());

  if resolved.is_ok() {
    return ResolveResult {
      found: true,
      path: String::from(resolved.ok().unwrap().to_str().unwrap()),
    };
  }

  let paths_map = config.compiler_options.clone().unwrap().paths;

  if paths_map.is_none() {
    return ResolveResult {
      found: false,
      path: String::from(""),
    };
  }

  // Iter paths to do full path match
  for (path_pattern, dest_paths) in paths_map.unwrap() {
    let star_match = match_star(path_pattern.clone(), source.clone());

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
