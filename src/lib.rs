use std::collections::HashMap;

/// Represents a single choice in the dialogue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Choice {
    pub text: String,
    pub target_node: String,
    pub condition: Option<String>,
}

/// Represents an action to be executed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Action {
    pub command: String,
}

/// Represents a single dialogue node.
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub name: String,
    pub meta: HashMap<String, String>,
    pub actions: Vec<Action>,
    pub body: String,
    pub choices: Vec<Choice>,
}

/// Represents the entire parsed dialogue.
#[derive(Debug, Clone, PartialEq)]
pub struct Dialogue {
    pub nodes: HashMap<String, Node>,
}

impl Dialogue {
    pub fn new() -> Self {
        Dialogue {
            nodes: HashMap::new(),
        }
    }
}

impl Default for Dialogue {
    fn default() -> Self {
        Self::new()
    }
}

/// Parses a Varion script string into a Dialogue struct.
///
/// # Arguments
///
/// * `script` - A string slice that holds the Varion script.
///
/// # Returns
///
/// * `Ok(Dialogue)` if parsing is successful.
/// * `Err(String)` with an error message if parsing fails.
pub fn parse(script: &str) -> Result<Dialogue, String> {
    let mut dialogue = Dialogue::new();
    let mut current_node: Option<Node> = None;
    let mut pending_condition: Option<String> = None;

    for (line_num, line) in script.lines().enumerate() {
        let trimmed_line = line.trim();
        let line_err = |msg: &str| format!("Error on line {}: {}", line_num + 1, msg);

        if trimmed_line.is_empty() {
            continue;
        }

        if trimmed_line.starts_with("::") {
            if let Some(node) = current_node.take() {
                dialogue.nodes.insert(node.name.clone(), node);
            }
            if pending_condition.is_some() {
                return Err(line_err("Dangling @if condition before new node."));
            }

            let node_name = trimmed_line[2..].trim().to_string();
            if node_name.is_empty() {
                return Err(line_err(
                    "Node declaration '::' must be followed by a name.",
                ));
            }
            current_node = Some(Node {
                name: node_name,
                meta: HashMap::new(),
                actions: Vec::new(),
                body: String::new(),
                choices: Vec::new(),
            });
            pending_condition = None; // Reset for new node
        } else if trimmed_line.starts_with("@if") {
            if current_node.is_none() {
                return Err(line_err("@if condition found outside of a node."));
            }
            if pending_condition.is_some() {
                return Err(line_err("Consecutive @if conditions are not allowed."));
            }
            pending_condition = Some(trimmed_line[3..].trim().to_string());
        } else if let Some(node) = &mut current_node {
            if let Some(meta_line) = trimmed_line.strip_prefix('@') {
                if pending_condition.is_some() {
                    return Err(line_err(
                        "@if must be immediately followed by a choice, not a meta/action line.",
                    ));
                }
                if let Some(action_str) = meta_line.strip_prefix("action:") {
                    node.actions.push(Action {
                        command: action_str.trim().to_string(),
                    });
                } else if let Some(colon_index) = meta_line.find(':') {
                    let key = meta_line[..colon_index].trim().to_string();
                    let value = meta_line[colon_index + 1..].trim().to_string();
                    node.meta.insert(key, value);
                } else {
                    return Err(line_err(&format!(
                        "Invalid meta or action line: {}",
                        trimmed_line
                    )));
                }
            } else if let Some(choice_line) = trimmed_line.strip_prefix('*') {
                let parts: Vec<&str> = choice_line.split("=>").map(|s| s.trim()).collect();
                if parts.len() != 2 {
                    return Err(line_err(&format!("Invalid choice format: {}", choice_line)));
                }

                let text = parts[0].to_string();
                let rest = parts[1];

                let (target_node_str, same_line_condition) = if let Some(if_index) = rest.find("@if")
                {
                    let target = rest[..if_index].trim().to_string();
                    let cond = rest[if_index + 3..].trim().to_string();
                    (target, Some(cond))
                } else {
                    (rest.to_string(), None)
                };

                if same_line_condition.is_some() && pending_condition.is_some() {
                    return Err(line_err(
                        "A choice cannot have both a preceding @if and an inline @if.",
                    ));
                }

                let final_condition = same_line_condition.or_else(|| pending_condition.take());

                node.choices.push(Choice {
                    text,
                    target_node: target_node_str,
                    condition: final_condition,
                });
            } else {
                if pending_condition.is_some() {
                    return Err(line_err(
                        "@if must be immediately followed by a choice, not body text.",
                    ));
                }
                if !node.body.is_empty() {
                    node.body.push('\n');
                }
                node.body.push_str(line);
            }
        } else {
            return Err(line_err(
                "Content found outside of a node declaration. Every line must belong to a node starting with '::'.",
            ));
        }
    }

    if let Some(node) = current_node.take() {
        if pending_condition.is_some() {
            return Err("Dangling @if condition at end of file.".to_string());
        }
        dialogue.nodes.insert(node.name.clone(), node);
    }

    Ok(dialogue)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_parse_simple_node() {
        let script = r#"
::start
@who: NPC
Hello, world!
* Go next => next_node
        "#;
        let dialogue = parse(script).unwrap();
        assert_eq!(dialogue.nodes.len(), 1);
        let node = dialogue.nodes.get("start").unwrap();
        assert_eq!(node.name, "start");
        assert_eq!(node.meta.get("who").unwrap(), "NPC");
        assert_eq!(node.body.trim(), "Hello, world!");
        assert_eq!(node.choices.len(), 1);
        assert_eq!(node.choices[0].text, "Go next");
        assert_eq!(node.choices[0].target_node, "next_node");
        assert_eq!(node.choices[0].condition, None);
    }

    #[test]
    fn test_parse_full_example() {
        let script = r#"
::start
@background: images/bg.png
@who: NPC
@action: set help_requested = 0

Welcome! What can I do for you?

* I need help! => ask_help
* Just looking. => end_neutral

::ask_help
@who: NPC
@action: set help_requested = 1

Really? I would be grateful for your help!

* Yes, I'll help. => help_accepted
* Sorry, I'm busy. => help_declined @if reputation < 3
        "#;

        let dialogue = parse(script).unwrap();
        assert_eq!(dialogue.nodes.len(), 2);

        let start_node = dialogue.nodes.get("start").unwrap();
        assert_eq!(start_node.name, "start");
        assert_eq!(start_node.meta.get("who").unwrap(), "NPC");
        assert_eq!(start_node.actions.len(), 1);
        assert_eq!(start_node.actions[0].command, "set help_requested = 0");
        assert_eq!(start_node.body.trim(), "Welcome! What can I do for you?");
        assert_eq!(start_node.choices.len(), 2);
        assert_eq!(start_node.choices[0].target_node, "ask_help");

        let ask_help_node = dialogue.nodes.get("ask_help").unwrap();
        assert_eq!(ask_help_node.name, "ask_help");
        assert_eq!(ask_help_node.choices.len(), 2);
        let conditional_choice = &ask_help_node.choices[1];
        assert_eq!(conditional_choice.text, "Sorry, I'm busy.");
        assert_eq!(conditional_choice.target_node, "help_declined");
        assert_eq!(conditional_choice.condition, Some("reputation < 3".to_string()));
    }

    #[test]
    fn test_parse_multiline_body() {
        let script = r#"
::multiline
This is the first line.
    This is the second line, with indentation.
* A choice => somewhere
        "#;
        let dialogue = parse(script).unwrap();
        let node = dialogue.nodes.get("multiline").unwrap();
        assert_eq!(node.body, "This is the first line.\n    This is the second line, with indentation.");
    }
    
    #[test]
    fn test_no_node_error() {
        let script = "Just some text without a node.";
        assert!(parse(script).is_err());
    }

    #[test]
    fn test_parse_varion_examples_va() {
        let path = "examples/varion_examples.va";
        let script = fs::read_to_string(path).expect("Should have been able to read the file");
        let result = parse(&script);
        assert!(result.is_ok(), "Parsing failed with: {:?}", result.err());
        let dialogue = result.unwrap();
        assert_eq!(dialogue.nodes.len(), 2);
        assert!(dialogue.nodes.contains_key("start_example"));
        assert!(dialogue.nodes.contains_key("end_example"));
    }

    #[test]
    fn test_parse_varion_long_example_vion() {
        let path = "examples/varion_long_example.vion";
        let script = fs::read_to_string(path).expect("Should have been able to read the file");
        let result = parse(&script);
        assert!(result.is_ok(), "Parsing failed with: {:?}", result.err());
        let dialogue = result.unwrap();
        assert_eq!(dialogue.nodes.len(), 4);
        assert!(dialogue.nodes.contains_key("start"));
        assert!(dialogue.nodes.contains_key("end_final"));
        let ask_for_reward_node = dialogue.nodes.get("ask_for_reward").unwrap();
        assert_eq!(ask_for_reward_node.choices.len(), 3);
        let offer_help_node = dialogue.nodes.get("offer_help").unwrap();
        assert_eq!(offer_help_node.actions.len(), 1);
    }
}
