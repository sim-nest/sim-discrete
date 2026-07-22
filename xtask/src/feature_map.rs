use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, PartialEq, Eq)]
struct PackageFeatures {
    package: String,
    features: Vec<String>,
}

pub fn check(repo_root: &Path) -> Result<(), String> {
    let packages = packages_with_declared_features(repo_root)?;
    if packages.is_empty() {
        return Ok(());
    }

    let feature_map_path = repo_root.join("docs/generated/feature-map.json");
    let feature_map = fs::read_to_string(&feature_map_path).map_err(|err| {
        format!(
            "{}: generated feature map is required when packages declare features: {err}",
            feature_map_path.display()
        )
    })?;
    validate_feature_map(&feature_map, &packages)
}

fn packages_with_declared_features(repo_root: &Path) -> Result<Vec<PackageFeatures>, String> {
    let members = workspace_members(repo_root)?;
    let mut packages = Vec::new();
    for member in members {
        let manifest = repo_root.join(member).join("Cargo.toml");
        let text = fs::read_to_string(&manifest)
            .map_err(|err| format!("{}: read manifest: {err}", manifest.display()))?;
        let Some(package) = package_name(&text) else {
            continue;
        };
        let features = declared_features(&text);
        if !features.is_empty() {
            packages.push(PackageFeatures { package, features });
        }
    }
    Ok(packages)
}

fn workspace_members(repo_root: &Path) -> Result<Vec<PathBuf>, String> {
    let manifest = repo_root.join("Cargo.toml");
    let text = fs::read_to_string(&manifest)
        .map_err(|err| format!("{}: read workspace manifest: {err}", manifest.display()))?;
    let mut members = Vec::new();
    let mut in_members = false;
    for line in text.lines() {
        let trimmed = strip_comment(line).trim();
        if !in_members {
            if trimmed.starts_with("members") && trimmed.contains('[') {
                in_members = true;
            } else {
                continue;
            }
        }
        members.extend(quoted_strings(trimmed).into_iter().map(PathBuf::from));
        if trimmed.contains(']') {
            break;
        }
    }
    Ok(members)
}

fn package_name(manifest: &str) -> Option<String> {
    let mut in_package = false;
    for line in manifest.lines() {
        let trimmed = strip_comment(line).trim();
        match section(trimmed) {
            Some("package") => {
                in_package = true;
                continue;
            }
            Some(_) => {
                in_package = false;
                continue;
            }
            None => {}
        }
        if in_package && trimmed.starts_with("name") {
            return quoted_strings(trimmed).into_iter().next();
        }
    }
    None
}

fn declared_features(manifest: &str) -> Vec<String> {
    let mut features = BTreeSet::new();
    let mut in_features = false;
    for line in manifest.lines() {
        let trimmed = strip_comment(line).trim();
        match section(trimmed) {
            Some("features") => {
                in_features = true;
                continue;
            }
            Some(_) => {
                in_features = false;
                continue;
            }
            None => {}
        }
        if !in_features || trimmed.is_empty() {
            continue;
        }
        if let Some((name, _)) = trimmed.split_once('=') {
            features.insert(name.trim().trim_matches('"').to_owned());
        }
    }
    features.into_iter().collect()
}

fn validate_feature_map(feature_map: &str, packages: &[PackageFeatures]) -> Result<(), String> {
    let objects = package_objects(feature_map);
    let mut missing = Vec::new();
    for package in packages {
        let Some(object) = objects.get(&package.package) else {
            missing.push(format!("{} missing from feature-map.json", package.package));
            continue;
        };
        if object.contains("\"features\": []") {
            missing.push(format!(
                "{} declares features but feature-map.json has none",
                package.package
            ));
            continue;
        }
        for feature in &package.features {
            let expected = format!("\"name\": \"{feature}\"");
            if !object.contains(&expected) {
                missing.push(format!(
                    "{} feature {feature} missing from feature-map.json",
                    package.package
                ));
            }
        }
    }

    if missing.is_empty() {
        Ok(())
    } else {
        Err(missing.join("; "))
    }
}

fn package_objects(feature_map: &str) -> BTreeMap<String, String> {
    let mut objects = BTreeMap::new();
    let mut depth = 0usize;
    let mut start = None;
    for (index, byte) in feature_map.bytes().enumerate() {
        match byte {
            b'{' => {
                if depth == 1 {
                    start = Some(index);
                }
                depth += 1;
            }
            b'}' => {
                depth = depth.saturating_sub(1);
                if depth == 1
                    && let Some(start_index) = start.take()
                {
                    let object = &feature_map[start_index..=index];
                    if let Some(package) = object_package(object) {
                        objects.insert(package, object.to_owned());
                    }
                }
            }
            _ => {}
        }
    }
    objects
}

fn object_package(object: &str) -> Option<String> {
    let package_index = object.rfind("\"package\"")?;
    let after_package = &object[package_index..];
    quoted_strings(after_package).into_iter().nth(1)
}

fn section(line: &str) -> Option<&str> {
    line.strip_prefix('[')?.strip_suffix(']')
}

fn strip_comment(line: &str) -> &str {
    line.split_once('#').map_or(line, |(before, _)| before)
}

fn quoted_strings(line: &str) -> Vec<String> {
    let mut strings = Vec::new();
    let mut current = String::new();
    let mut in_string = false;
    let mut escaped = false;
    for ch in line.chars() {
        if escaped {
            current.push(ch);
            escaped = false;
            continue;
        }
        match ch {
            '\\' if in_string => escaped = true,
            '"' if in_string => {
                strings.push(std::mem::take(&mut current));
                in_string = false;
            }
            '"' => in_string = true,
            _ if in_string => current.push(ch),
            _ => {}
        }
    }
    strings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_declared_feature_names() {
        let features = declared_features(
            r#"
[package]
name = "demo"

[features]
default = []
runtime = ["dep:core"]

[dependencies]
core = { version = "1", features = ["std"] }
"#,
        );
        assert_eq!(features, vec!["default", "runtime"]);
    }

    #[test]
    fn validates_featureful_package_coverage() {
        let packages = [PackageFeatures {
            package: "demo".to_owned(),
            features: vec!["default".to_owned(), "runtime".to_owned()],
        }];
        let feature_map = r#"
{
  "packages": [
    {
      "features": [
        { "name": "default" },
        { "name": "runtime" }
      ],
      "package": "demo"
    }
  ]
}
"#;
        validate_feature_map(feature_map, &packages).unwrap();
    }

    #[test]
    fn ignores_nested_workspace_edge_package_names() {
        let packages = [PackageFeatures {
            package: "demo".to_owned(),
            features: vec!["runtime".to_owned()],
        }];
        let feature_map = r#"
{
  "packages": [
    {
      "features": [
        {
          "name": "runtime",
          "workspace_edges": [
            { "package": "dependency" }
          ]
        }
      ],
      "package": "demo"
    }
  ]
}
"#;
        validate_feature_map(feature_map, &packages).unwrap();
    }

    #[test]
    fn rejects_empty_feature_map_for_featureful_package() {
        let packages = [PackageFeatures {
            package: "demo".to_owned(),
            features: vec!["runtime".to_owned()],
        }];
        let err = validate_feature_map(
            r#"{ "packages": [{ "features": [], "package": "demo" }] }"#,
            &packages,
        )
        .unwrap_err();
        assert!(err.contains("demo declares features"));
    }
}
