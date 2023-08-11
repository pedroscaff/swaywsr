use itertools::Itertools;
use std::collections::HashMap as Map;
use swayipc::{
    Connection, Node, NodeType, WindowChange, WindowEvent, WorkspaceChange, WorkspaceEvent,
};

pub mod config;
pub mod icons;

pub struct Config {
    pub icons: Map<String, char>,
    pub aliases: Map<String, String>,
    pub general: Map<String, String>,
    pub options: Map<String, bool>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            icons: icons::NONE.clone(),
            aliases: config::EMPTY_MAP.clone(),
            general: config::EMPTY_MAP.clone(),
            options: config::EMPTY_OPT_MAP.clone(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum LookupError {
    #[error("Failed to get app_id or window_properties for node: {0:#?}")]
    MissingInformation(String),
    #[error("Failed to get name for workspace: {0:#?}")]
    WorkspaceName(Box<Node>),
}

fn get_option(config: &Config, key: &str) -> bool {
    config.options.get(key).map_or(false, |v| *v)
}

fn get_class(node: &Node, config: &Config) -> Result<String, LookupError> {
    let name = {
        match &node.app_id {
            Some(id) => Some(id.to_owned()),
            None => node
                .window_properties
                .as_ref()
                .and_then(|p| p.class.as_ref())
                .map(|p| p.to_owned()),
        }
    };

    if let Some(class) = name {
        let class_display_name = match config.aliases.get(&class) {
            Some(alias) => alias,
            None => &class,
        };

        let no_names = get_option(config, "no_names");

        Ok(match config.icons.get(&class) {
            Some(icon) => {
                if no_names {
                    format!("{}", icon)
                } else {
                    format!("{} {}", icon, class_display_name)
                }
            }
            None => match config.general.get("default_icon") {
                Some(default_icon) => {
                    if no_names {
                        default_icon.to_string()
                    } else {
                        format!("{} {}", default_icon, class_display_name)
                    }
                }
                None => class_display_name.to_string(),
            },
        })
    } else {
        Err(LookupError::MissingInformation(format!("{:?}", node)))
    }
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
            window_nodes.push(n);
        }
    }

    window_nodes
}

/// Return a collection of window classes
fn get_classes(workspace: &Node, config: &Config) -> Vec<String> {
    let window_nodes = {
        let mut f = get_window_nodes(vec![workspace.floating_nodes.iter().collect()]);
        let mut n = get_window_nodes(vec![workspace.nodes.iter().collect()]);
        n.append(&mut f);
        n
    };

    let mut window_classes = Vec::new();
    for node in window_nodes {
        let class = match get_class(node, config) {
            Ok(class) => class,
            Err(e) => {
                eprintln!("get class error: {e:?}");
                continue;
            }
        };
        window_classes.push(class);
    }

    window_classes
}

/// Update all workspace names in tree
pub fn update_tree(connection: &mut Connection, config: &Config) -> anyhow::Result<()> {
    let tree = connection.get_tree()?;
    for workspace in get_workspaces(tree) {
        let separator = match config.general.get("separator") {
            Some(s) => s,
            None => " | ",
        };

        let classes = get_classes(&workspace, config);
        let classes = if get_option(config, "remove_duplicates") {
            classes.into_iter().unique().collect()
        } else {
            classes
        };

        let classes = classes.join(separator);
        let classes = if !classes.is_empty() {
            format!(" {}", classes)
        } else {
            classes
        };

        let old: String = workspace
            .name
            .to_owned()
            .ok_or_else(|| LookupError::WorkspaceName(Box::new(workspace)))?;

        let mut new = old.split(' ').next().unwrap().to_owned();

        if !classes.is_empty() {
            new.push_str(&classes);
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
    config: &Config,
) -> anyhow::Result<()> {
    match event.change {
        WindowChange::New | WindowChange::Close | WindowChange::Move => {
            update_tree(connection, config)
        }
        _ => Ok(()),
    }
}

pub fn handle_workspace_event(
    event: &WorkspaceEvent,
    connection: &mut Connection,
    config: &Config,
) -> anyhow::Result<()> {
    match event.change {
        WorkspaceChange::Empty | WorkspaceChange::Focus => update_tree(connection, config),
        _ => Ok(()),
    }
}
