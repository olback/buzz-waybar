use {serde::Serialize, std::collections::HashMap};

#[derive(Debug, Serialize)]
pub struct ReturnData {
    pub text: String,
    pub alt: String,
    pub percentage: u8,
    pub tooltip: String,
}

impl ReturnData {
    pub fn new(data: &HashMap<String, usize>) -> Self {
        match data.iter().map(|v| v.1).sum() {
            0 => Self {
                text: "0 unread".to_string(),
                alt: "0".to_string(),
                percentage: 0,
                tooltip: "No unread emails".to_string(),
            },
            n => Self {
                text: format!("{} unread", n),
                alt: n.to_string(),
                percentage: 100,
                tooltip: {
                    let mut tooltip_lines = Vec::<String>::with_capacity(data.len() + 1);
                    tooltip_lines.push(format!(
                        "<b>You have {} unread {}</b>",
                        n,
                        if n > 1 { "emails" } else { "email" }
                    ));
                    for (name, unread) in data {
                        tooltip_lines.push(format!("{}: {}", name, unread));
                    }
                    tooltip_lines
                }
                .join("\n"),
            },
        }
    }
}
