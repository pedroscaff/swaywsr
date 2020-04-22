#[macro_use]
extern crate failure_derive;
extern crate failure;
use failure::Error;

extern crate serde;

#[macro_use]
extern crate lazy_static;

extern crate toml;

use swayipc::reply::{Node, NodeType, WindowChange, WindowEvent, WorkspaceChange, WorkspaceEvent};
use swayipc::Connection;

use std::collections::HashMap as Map;
use std::collections::HashSet;

pub mod config;
pub mod icons;

pub struct Options {
    pub icons: Map<String, char>,
    pub aliases: Map<String, String>,
    pub general: Map<String, String>,
    pub names: bool,
    pub no_dupes: bool,
}

#[derive(Debug, Fail)]
enum LookupError {
    #[fail(
        display = "Failed to get app_id or window_properties for node: {:#?}",
        _0
    )]
    MissingInformation(String),
    #[fail(display = "Failed to get name for workspace: {:#?}", _0)]
    WorkspaceName(Box<Node>),
}

fn get_class(node: &Node) -> Result<String, LookupError> {
    let name = {
        match &node.app_id {
            Some(id) => Some(id.to_owned()),
            None => match &node.window_properties {
                Some(properties) => Some(properties.class.to_owned()),
                None => None,
            },
        }
    };
    if let Some(class) = name {
        Ok(format!("{}", &class))
    } else {
        Err(LookupError::MissingInformation(format!("{:?}", node)))
    }
}

fn get_icons(classes: &Vec<String>, options: &Options) -> Result<Vec<String>, LookupError> {
    let mut iconised_classes = Vec::new();
    for class in classes {
        iconised_classes.push(get_icon(class.to_string(), options)?);
    }
    Ok(iconised_classes)
}

fn get_icon(class_name: String, options: &Options) -> Result<String, LookupError> {
    let iconised_name = {
        match options.icons.get(&class_name) {
            Some(icon) => {
                if options.names {
                    format!("{} {}", icon, class_name)
                } else {
                    format!("{}", icon)
                }
            }
            None => format!("{}", class_name),
        }
    };
    Ok(iconised_name.to_string())
}

fn get_aliases(classes: &Vec<String>, options: &Options) -> Result<Vec<String>, LookupError> {
    let mut aliased_classes = Vec::new();
    for class in classes {
        aliased_classes.push(get_alias(class.to_string(), options)?);
    }
    Ok(aliased_classes)
}

fn get_alias(class_name: String, options: &Options) -> Result<String, LookupError> {
    let class_display_name = match options.aliases.get(&class_name) {
            Some(alias) => alias,
            None => &class_name,
        };
        Ok(format!("{}", class_display_name))
}

/// return a collection of workspace nodes
fn get_workspaces(tree: Node) -> Vec<Node> {
    let mut out = Vec::new();

    for output in tree.nodes {
        for container in output.nodes {
            if let NodeType::Workspace = container.node_type {
                out.push(container);
            }
        }
    }

    out
}

/// get all nodes for any depth collection of nodes
fn get_window_nodes(mut nodes: Vec<Vec<&Node>>) -> Vec<&Node> {
    let mut window_nodes = Vec::new();

    while let Some(next) = nodes.pop() {
        for n in next {
            nodes.push(n.nodes.iter().collect());
            if let Some(_) = n.window {
                window_nodes.push(n);
            } else if let Some(_) = n.app_id {
                window_nodes.push(n);
            }
        }
    }

    window_nodes
}

/// Return a collection of window classes
fn get_classes(workspace: &Node, options: &Options) -> Result<Vec<String>, Error> {
    let window_nodes = {
        let mut f = get_window_nodes(vec![workspace.floating_nodes.iter().collect()]);
        let mut n = get_window_nodes(vec![workspace.nodes.iter().collect()]);
        n.append(&mut f);
        n
    };

    let mut window_classes = Vec::new();
    for node in window_nodes {
        window_classes.push(get_class(node)?);
    }

    if options.no_dupes {
        let mut unique = HashSet::new();
        window_classes.retain(|class| unique.insert(class.clone()));
        window_classes.sort();
    }
    Ok(window_classes)
}

/// Update all workspace names in tree
pub fn update_tree(connection: &mut Connection, options: &Options) -> Result<(), Error> {
    let tree = connection.get_tree()?;
    for workspace in get_workspaces(tree) {
        let separator = match options.general.get("separator") {
            Some(s) => s,
            None => " | ",
        };

        let classes = get_classes(&workspace, options)?;
        let iconised_classes = get_icons(&classes, options)?;
        let aliased_classes = get_aliases(&iconised_classes, options)?.join(separator);
        let renamed_classes = if !aliased_classes.is_empty() {
            format!(" {}", aliased_classes)
        } else {
            aliased_classes
        };

        let old: String = workspace
            .name
            .to_owned()
            .ok_or_else(|| LookupError::WorkspaceName(Box::new(workspace)))?;

        let mut new = old.split(' ').next().unwrap().to_owned();

        if !classes.is_empty() {
            new.push_str(&renamed_classes);
        }

        if old != new {
            let command = format!("rename workspace \"{}\" to \"{}\"", old, new);
            connection.run_command(&command)?;
        }
    }
    Ok(())
}

pub fn handle_window_event(
    event: &WindowEvent,
    connection: &mut Connection,
    options: &Options,
) -> Result<(), Error> {
    match event.change {
        WindowChange::New | WindowChange::Close | WindowChange::Move => {
            update_tree(connection, options)
        }
        _ => Ok(()),
    }
}

pub fn handle_workspace_event(
    event: &WorkspaceEvent,
    connection: &mut Connection,
    options: &Options,
) -> Result<(), Error> {
    match event.change {
        WorkspaceChange::Empty | WorkspaceChange::Focus => update_tree(connection, options),
        _ => Ok(()),
    }
}
