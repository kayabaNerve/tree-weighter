fn main() {
    let data = std::fs::read_to_string(std::env::args().skip(1).next().unwrap())
        .unwrap()
        .parse::<toml::Table>()
        .unwrap()["package"]
        .clone();

    let sanitize = |str: &str| str.split(" ").next().unwrap().to_string();

    let mut amount_of_immediate_parents = std::collections::HashMap::<String, usize>::new();
    let mut immediate_dependencies = std::collections::HashMap::<String, Vec<String>>::new();
    for package in data.as_array().unwrap() {
        if !amount_of_immediate_parents.contains_key(&sanitize(package["name"].as_str().unwrap())) {
            amount_of_immediate_parents.insert(sanitize(package["name"].as_str().unwrap()), 0);
        }
        if !package.as_table().unwrap().contains_key("dependencies") {
            immediate_dependencies.insert(sanitize(package["name"].as_str().unwrap()), vec![]);
            continue;
        }
        immediate_dependencies.insert(
            sanitize(package["name"].as_str().unwrap()),
            package["dependencies"]
                .as_array()
                .unwrap()
                .iter()
                .map(|value| sanitize(value.as_str().unwrap()))
                .collect(),
        );
        for dependency in package["dependencies"].as_array().unwrap() {
            let new_val = amount_of_immediate_parents
                .get(&sanitize(dependency.as_str().unwrap()))
                .cloned()
                .unwrap_or(0)
                + 1;
            amount_of_immediate_parents.insert(sanitize(dependency.as_str().unwrap()), new_val);
        }
    }
    assert_eq!(
        amount_of_immediate_parents.len(),
        immediate_dependencies.len(),
    );

    fn build_dependency_tree(
        immediate_dependencies: &std::collections::HashMap<String, Vec<String>>,
        existing_set: &mut std::collections::HashMap<String, usize>,
        parent: String,
    ) {
        for dependency in &immediate_dependencies[&parent] {
            if !existing_set.contains_key(dependency) {
                let existing = existing_set.get(dependency).cloned().unwrap_or(0);
                existing_set.insert(dependency.to_string(), existing + 1);
                build_dependency_tree(immediate_dependencies, existing_set, dependency.to_string());
            }
        }
    }

    fn get_amount_of_dependencies(
        immediate_dependencies: &std::collections::HashMap<String, Vec<String>>,
        parent: String,
    ) -> usize {
        let mut existing_set = std::collections::HashMap::new();
        build_dependency_tree(immediate_dependencies, &mut existing_set, parent);
        existing_set.len()
    }

    let mut amount_of_parents = std::collections::HashMap::<String, usize>::new();
    for key in amount_of_immediate_parents.keys() {
        let mut children = std::collections::HashMap::new();
        build_dependency_tree(&immediate_dependencies, &mut children, key.clone());
        for child in children.keys() {
            let new_val = amount_of_parents
                .get(&sanitize(child.as_str()))
                .cloned()
                .unwrap_or(0)
                + 1;
            amount_of_parents.insert(sanitize(child.as_str()), new_val);
        }
    }

    fn get_unique_dependencies(
        immediate_dependencies: &std::collections::HashMap<String, Vec<String>>,
        amount_of_immediate_parents: &std::collections::HashMap<String, usize>,
        parent: String,
    ) -> Vec<String> {
        let mut existing_set = std::collections::HashMap::new();
        build_dependency_tree(immediate_dependencies, &mut existing_set, parent.clone());
        for (dependency, uses) in existing_set.clone() {
            if amount_of_immediate_parents[&dependency] != uses {
                existing_set.remove(&dependency);
                let mut child_set = std::collections::HashMap::new();
                build_dependency_tree(immediate_dependencies, &mut child_set, dependency.clone());
                for child in child_set {
                    existing_set.remove(&child.0);
                }
            }
        }
        existing_set.into_iter().map(|(x, _)| x).collect()
    }

    let mut list = vec![];
    for key in amount_of_immediate_parents.keys() {
        list.push((
            key,
            amount_of_immediate_parents[key],
            get_unique_dependencies(
                &immediate_dependencies,
                &amount_of_immediate_parents,
                key.clone(),
            ),
            {
                let unique = get_unique_dependencies(
                    &immediate_dependencies,
                    &amount_of_immediate_parents,
                    key.clone(),
                );
                let mut res = String::new();
                for package in &unique {
                    if !res.is_empty() {
                        res += ", ";
                    }
                    res += package;
                }
                res
            },
            get_amount_of_dependencies(&immediate_dependencies, key.to_string()),
        ));
    }
    list.sort_by(|a, b| {
        let mut res = a.1.partial_cmp(&b.1).unwrap();
        if res == std::cmp::Ordering::Equal {
            res = a.2.len().partial_cmp(&b.2.len()).unwrap().reverse();
            if res == std::cmp::Ordering::Equal {
                res = a.4.partial_cmp(&b.4).unwrap().reverse();
            }
        }
        res
    });

    while list.first().is_some() && (list.first().unwrap().1 == 0) {
        list.remove(0);
    }

    println!("Lockfile has {} crates.", list.len());

    let mut longest_crate_name = "Crate".len();
    for item in &list {
        if item.0.len() > longest_crate_name {
            longest_crate_name = item.0.len();
        }
    }
    let mut crate_header = "Crate".to_string();
    while crate_header.len() < longest_crate_name {
        crate_header += " ";
    }

    println!("--------------------------------------------------------------");
    println!(
        "| {} | Parents | Unique Dependencies | Total Dependencies |",
        crate_header
    );
    println!("--------------------------------------------------------------");
    for mut item in list.into_iter() {
        if item.1 > 3 {
            continue;
        }
        println!(
            "| {} | {} | {} | {} |",
            {
                let mut dependency = item.0.to_string();
                while dependency.len() < longest_crate_name {
                    dependency += " ";
                }
                dependency
            },
            {
                let mut res = format!("{}", item.1);
                while res.len() < "Parents".len() {
                    res += " ";
                }
                res
            },
            {
                while item.3.len() < "Unique Dependencies".len() {
                    item.3 += " ";
                }
                item.3
            },
            {
                let mut res = format!("{}", item.4);
                while res.len() < "Total Dependencies".len() {
                    res += " ";
                }
                res
            },
        );
    }
    println!("--------------------------------------------------------------");
}
